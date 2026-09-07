//! Deterministic interaction coordination. No resources, environment or clock.
use crate::{
    EditError, Editor, Error, Event, Prompt, Role,
    actions::{EditCommand, Input, Request},
    render::{Mutation, Renderer},
};
use std::ops::Range;

#[derive(Default)]
pub(crate) struct Effects {
    pub mutations: Vec<Mutation>,
    pub event: Option<Event>,
}
struct Surface {
    prompt: Prompt,
    size: (usize, usize),
    renderer: Renderer,
}
pub(crate) struct Engine {
    pub editor: Editor,
    surface: Option<Surface>,
}
impl Engine {
    pub fn new(editor: Editor) -> Self {
        Self {
            editor,
            surface: None,
        }
    }
    pub fn is_open(&self) -> bool {
        self.surface.is_some()
    }
    pub fn start(&mut self, prompt: Prompt, size: (usize, usize)) -> Result<Effects, Error> {
        if self.is_open() {
            return Err(Error::State);
        }
        if size.0 < 2 || size.1 < 2 {
            return Err(Error::UnsuitableTerminal);
        }
        self.surface = Some(Surface {
            prompt,
            size,
            renderer: Renderer::default(),
        });
        Ok(Effects {
            mutations: self.redraw(),
            event: None,
        })
    }
    fn redraw(&mut self) -> Vec<Mutation> {
        let s = self
            .surface
            .as_mut()
            .expect("active surface checked by caller");
        s.renderer.redraw(&self.editor, &s.prompt, s.size)
    }
    pub fn apply(&mut self, input: Input) -> Result<Effects, Error> {
        if !self.is_open() {
            return Err(Error::State);
        }
        let mut mutations = Vec::new();
        match input {
            Input::Text(text) => {
                if let Err(error) = self.editor.insert(&text) {
                    return Ok(Effects {
                        event: Some(Event::Rejected(error)),
                        ..Effects::default()
                    });
                }
            }
            Input::Edit(command) => match command {
                EditCommand::Left => self.editor.left(),
                EditCommand::Right => self.editor.right(),
                EditCommand::Home => self.editor.home(),
                EditCommand::End => self.editor.end(),
                EditCommand::Backspace => self.editor.backspace(),
                EditCommand::Delete => self.editor.delete(),
                EditCommand::HistoryPrevious => self.editor.history_up(),
                EditCommand::HistoryNext => self.editor.history_down(),
            },
            Input::Request(Request::Submit) => {
                return Ok(self.finish(Event::Submitted(self.editor.text().into())));
            }
            Input::Request(Request::Interrupt) => return Ok(self.finish(Event::Interrupted)),
            Input::TransportEof => return Ok(self.finish(Event::EndOfInput)),
            Input::Request(Request::DeleteOrEof) if self.editor.text().is_empty() => {
                return Ok(self.finish(Event::EndOfInput));
            }
            Input::Request(Request::DeleteOrEof) => self.editor.delete(),
            Input::Request(Request::Completion) => {
                return Ok(Effects {
                    event: Some(Event::CompletionRequested),
                    ..Effects::default()
                });
            }
            Input::Request(Request::Redraw) => {
                mutations.extend(self.surface.as_mut().unwrap().renderer.clear())
            }
            Input::Resize(columns, rows) => {
                if columns < 2 || rows < 2 {
                    return Err(Error::UnsuitableTerminal);
                }
                let s = self.surface.as_mut().unwrap();
                if s.size == (columns, rows) {
                    return Ok(Effects::default());
                }
                s.size = (columns, rows);
            }
            Input::Rejected(error) => {
                return Ok(Effects {
                    event: Some(Event::Rejected(error)),
                    ..Effects::default()
                });
            }
        }
        mutations.extend(self.redraw());
        Ok(Effects {
            mutations,
            event: None,
        })
    }
    fn finish(&mut self, event: Event) -> Effects {
        let mut surface = self.surface.take().unwrap();
        Effects {
            mutations: surface.renderer.leave(event == Event::Interrupted),
            event: Some(event),
        }
    }
    pub fn close(&mut self) -> Effects {
        Effects {
            mutations: self
                .surface
                .take()
                .map_or_else(Vec::new, |mut s| s.renderer.leave(false)),
            event: None,
        }
    }
    pub fn abandon(&mut self) {
        self.surface = None;
    }
    pub fn complete(&mut self, range: Range<usize>, text: &str) -> Result<Effects, Error> {
        if !self.is_open() {
            return Err(Error::State);
        }
        self.editor.replace(range, text)?;
        Ok(Effects {
            mutations: self.redraw(),
            event: None,
        })
    }
    pub fn external_output(&mut self, role: Role, text: &str) -> Result<Effects, Error> {
        if !self.is_open() {
            return Err(Error::State);
        }
        let text = text.replace("\r\n", "\n");
        if !crate::core::valid_text(&text) {
            return Err(EditError::InvalidText.into());
        }
        let mut mutations = vec![Mutation::Paste(false)];
        mutations.extend(self.surface.as_mut().unwrap().renderer.erase());
        mutations.push(Mutation::Style(role));
        for (i, line) in text.split('\n').enumerate() {
            if i > 0 {
                mutations.push(Mutation::Newline);
            }
            if !line.is_empty() {
                mutations.push(Mutation::Text(line.into()));
            }
        }
        mutations.push(Mutation::Style(Role::Default));
        if !text.ends_with('\n') {
            mutations.push(Mutation::Newline);
        }
        mutations.push(Mutation::Paste(true));
        mutations.extend(self.redraw());
        Ok(Effects {
            mutations,
            event: None,
        })
    }
}
