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
    dirty: bool,
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
            dirty: true,
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
        s.dirty = false;
        s.renderer.redraw(&self.editor, &s.prompt, s.size)
    }
    pub fn apply(&mut self, input: Input) -> Result<Effects, Error> {
        self.apply_inner(input, false)
    }
    /// Apply already-ready input without rebuilding intermediate presentations.
    /// Host events, explicit redraw and lifecycle transitions always flush first.
    pub fn apply_deferred(&mut self, input: Input) -> Result<Effects, Error> {
        self.apply_inner(input, true)
    }
    pub fn flush(&mut self) -> Effects {
        Effects {
            mutations: if self.surface.as_ref().is_some_and(|s| s.dirty) {
                self.redraw()
            } else {
                Vec::new()
            },
            event: None,
        }
    }
    fn observable(&mut self, event: Event) -> Effects {
        let mut effects = self.flush();
        effects.event = Some(event);
        effects
    }
    fn apply_inner(&mut self, input: Input, deferred: bool) -> Result<Effects, Error> {
        if !self.is_open() {
            return Err(Error::State);
        }
        let mut mutations = Vec::new();
        let force = matches!(input, Input::Request(Request::Redraw) | Input::Resize(..));
        match input {
            Input::Text(text) => {
                if let Err(error) = self.editor.insert(&text) {
                    return Ok(self.observable(Event::Rejected(error)));
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
                return Ok(self.observable(Event::CompletionRequested));
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
                return Ok(self.observable(Event::Rejected(error)));
            }
        }
        self.surface.as_mut().unwrap().dirty = true;
        if !deferred || force {
            mutations.extend(self.redraw());
        }
        Ok(Effects {
            mutations,
            event: None,
        })
    }
    fn finish(&mut self, event: Event) -> Effects {
        let mut effects = self.flush();
        let mut surface = self.surface.take().unwrap();
        effects
            .mutations
            .extend(surface.renderer.leave(event == Event::Interrupted));
        effects.event = Some(event);
        effects
    }
    pub fn close(&mut self) -> Effects {
        let mut effects = self.flush();
        if let Some(mut s) = self.surface.take() {
            effects.mutations.extend(s.renderer.leave(false));
        }
        effects
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
