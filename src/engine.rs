//! Deterministic interaction coordination. No resources, environment or clock.
use crate::{
    Editor, Error, Event, Prompt, Role,
    actions::{EditCommand, Input, Request},
    render::{Damage, Mutation, Renderer},
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
    damage: Damage,
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
            damage: Damage::Rebuild,
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
        let damage = if s.dirty { s.damage } else { Damage::Rebuild };
        s.dirty = false;
        s.renderer
            .redraw_changed(&self.editor, &s.prompt, s.size, damage)
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
        let previous_len = self.editor.text().len();
        let at_end = self.editor.cursor() == previous_len;
        let ascii_tail = self
            .editor
            .text()
            .as_bytes()
            .last()
            .is_none_or(|b| *b == b' ' || b.is_ascii_graphic());
        let mut damage = match &input {
            Input::Edit(
                EditCommand::Left | EditCommand::Right | EditCommand::Home | EditCommand::End,
            ) => Damage::Cursor,
            Input::Text(text)
                if at_end
                    && ascii_tail
                    && text.bytes().all(|b| b == b' ' || b.is_ascii_graphic()) =>
            {
                Damage::AppendAscii
            }
            Input::Edit(EditCommand::Backspace) if at_end && ascii_tail => Damage::BackspaceAscii,
            _ => Damage::Rebuild,
        };
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
        if damage == Damage::BackspaceAscii && previous_len != self.editor.text().len() + 1 {
            damage = Damage::Rebuild;
        }
        let s = self.surface.as_mut().unwrap();
        if s.dirty && s.damage != damage {
            damage = Damage::Rebuild;
        }
        s.dirty = true;
        s.damage = damage;
        if !deferred || force {
            mutations.extend(self.redraw());
        }
        Ok(Effects {
            mutations,
            event: None,
        })
    }
    fn finish(&mut self, event: Event) -> Effects {
        self.editor.end_draft();
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
        self.surface.as_mut().unwrap().damage = Damage::Rebuild;
        Ok(Effects {
            mutations: self.redraw(),
            event: None,
        })
    }
    pub fn complete_at(
        &mut self,
        revision: crate::DraftRevision,
        range: Range<usize>,
        text: &str,
    ) -> Result<(crate::AnalysisOutcome, Effects), Error> {
        if !self.is_open() {
            return Err(Error::State);
        }
        if revision != self.editor.revision() {
            return Ok((crate::AnalysisOutcome::Stale, Effects::default()));
        }
        // Exclusive engine ownership keeps comparison, validation and mutation
        // one operation. Both completion paths use the same editor and redraw.
        Ok((crate::AnalysisOutcome::Applied, self.complete(range, text)?))
    }
    pub fn output_document(&mut self, document: &crate::Document) -> Result<Effects, Error> {
        let size = self.surface.as_ref().ok_or(Error::State)?.size;
        let mutations = document.mutations(size.0)?;
        Ok(self.output_mutations(mutations))
    }
    fn output_mutations(&mut self, content: Vec<Mutation>) -> Effects {
        let mut mutations = vec![Mutation::Paste(false)];
        mutations.extend(self.surface.as_mut().unwrap().renderer.erase());
        mutations.extend(content);
        mutations.push(Mutation::Paste(true));
        mutations.extend(self.redraw());
        Effects {
            mutations,
            event: None,
        }
    }
    pub fn external_output(&mut self, role: Role, text: &str) -> Result<Effects, Error> {
        if !self.is_open() {
            return Err(Error::State);
        }
        let mutations = crate::document::plain_mutations(role, text)?;
        Ok(self.output_mutations(mutations))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Theme, presentation::Frame, protocol::encode};

    #[test]
    fn structured_output_is_deterministic_and_rejection_does_not_mutate_surface() {
        use crate::{Document, Text, document::Block};
        let mut e = Engine::new(Editor::new(1024, 2));
        let doc = Document::new(vec![Block::Paragraph(Text::new("facts 界").unwrap())]).unwrap();
        assert!(matches!(e.output_document(&doc), Err(Error::State)));
        let prompt = Prompt::new("demo").unwrap();
        e.start(prompt.clone(), (2, 8)).unwrap();
        e.apply(Input::Text("a界b".into())).unwrap();
        e.apply(Input::Edit(EditCommand::Left)).unwrap();
        let huge = Document::new(vec![
            Block::Paragraph(Text::new(&"x".repeat(16384)).unwrap());
            64
        ])
        .unwrap();
        assert!(matches!(
            e.output_document(&huge),
            Err(Error::Edit(crate::EditError::Capacity))
        ));
        assert_eq!((e.editor.text(), e.editor.cursor()), ("a界b", 4));
        e.surface
            .as_ref()
            .unwrap()
            .renderer
            .assert_frame(&e.editor, &prompt, (2, 8));
        e.output_document(&doc).unwrap();
        assert_eq!((e.editor.text(), e.editor.cursor()), ("a界b", 4));
        e.surface
            .as_ref()
            .unwrap()
            .renderer
            .assert_frame(&e.editor, &prompt, (2, 8));
    }

    #[test]
    fn deferred_damage_completion_resize_and_output_invalidate_reuse() {
        for label in ["label", "界\u{301}", "👩\u{200d}💻"] {
            for width in [6, 20, 80] {
                let prompt = Prompt::new(label).unwrap();
                let mut e = Engine::new(Editor::new(4096, 1));
                e.start(prompt.clone(), (width, 8)).unwrap();
                for i in 0..60 {
                    e.apply_deferred(Input::Text("a".into())).unwrap();
                    e.apply_deferred(Input::Text("b".into())).unwrap();
                    e.flush();
                    e.apply_deferred(Input::Edit(EditCommand::Backspace))
                        .unwrap();
                    e.apply_deferred(Input::Edit(EditCommand::Backspace))
                        .unwrap();
                    e.flush();
                    e.surface.as_ref().unwrap().renderer.assert_frame(
                        &e.editor,
                        &prompt,
                        (width, 8),
                    );
                    e.apply_deferred(Input::Text("x".into())).unwrap();
                    // Completion must invalidate even a pending homogeneous append.
                    e.complete(
                        0..e.editor.text().len(),
                        if i % 2 == 0 { "界\na\u{301}" } else { "ascii" },
                    )
                    .unwrap();
                    e.surface.as_ref().unwrap().renderer.assert_frame(
                        &e.editor,
                        &prompt,
                        (width, 8),
                    );
                    e.external_output(Role::Dim, "notice\nnext").unwrap();
                    e.surface.as_ref().unwrap().renderer.assert_frame(
                        &e.editor,
                        &prompt,
                        (width, 8),
                    );
                    e.apply(Input::Resize(width + 1, 8)).unwrap();
                    e.surface.as_ref().unwrap().renderer.assert_frame(
                        &e.editor,
                        &prompt,
                        (width + 1, 8),
                    );
                    e.apply(Input::Resize(width, 8)).unwrap();
                }
            }
        }
    }

    #[test]
    fn cached_geometry_and_incremental_vt_match_fresh_layout_after_adversarial_edits() {
        let units = [
            "a",
            " ",
            "é",
            "界",
            "e\u{301}",
            "\u{301}",
            "👩\u{200d}💻",
            "\u{200d}",
            "♥\u{fe0f}",
            "🇮",
            "🇹",
            "\u{600}a",
            "\n",
            "\t",
        ];
        for width in [6, 9, 20, 80] {
            for ascii_only in [true, false] {
                let prompt = Prompt::new("p").unwrap();
                let theme = Theme::new(true, false, None);
                let mut e = Engine::new(Editor::new(2048, 3));
                e.editor.admit_history("old\n界 command").unwrap();
                let mut terminal = vt100::Parser::new(8, width as u16, 0);
                let initial = e.start(prompt.clone(), (width, 8)).unwrap();
                terminal.process(encode(&initial.mutations, theme).as_bytes());
                let mut random = 712_u64;
                for step in 0..4096 {
                    random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let choice = (random >> 32) as usize;
                    let input = match choice % 18 {
                        0 => Input::Edit(EditCommand::Home),
                        1 => Input::Edit(EditCommand::End),
                        2..=3 => Input::Edit(EditCommand::Left),
                        4 => Input::Edit(EditCommand::Right),
                        5..=6 => Input::Edit(EditCommand::Backspace),
                        7 => Input::Edit(EditCommand::Delete),
                        8 => Input::Edit(EditCommand::HistoryPrevious),
                        9 => Input::Edit(EditCommand::HistoryNext),
                        _ => Input::Text(if ascii_only {
                            "ab c"[(choice % 4)..][..1].into()
                        } else {
                            units[choice % units.len()].into()
                        }),
                    };
                    let effects = e.apply(input).unwrap();
                    terminal.process(encode(&effects.mutations, theme).as_bytes());
                    e.surface.as_ref().unwrap().renderer.assert_frame(
                        &e.editor,
                        &prompt,
                        (width, 8),
                    );
                    // vt100's joined-emoji width differs from REPLAI's frozen
                    // policy. The complete Frame equality above covers Unicode;
                    // this independent screen oracle scores ASCII transitions.
                    if ascii_only && !e.editor.text().contains('界') {
                        let mut fresh = vt100::Parser::new(8, width as u16, 0);
                        fresh.process(
                            encode(&Frame::new(&e.editor, &prompt, width, 8).draw(), theme)
                                .as_bytes(),
                        );
                        assert_eq!(
                            terminal.screen().contents(),
                            fresh.screen().contents(),
                            "step {step} width {width}"
                        );
                        assert_eq!(
                            terminal.screen().cursor_position(),
                            fresh.screen().cursor_position(),
                            "step {step} width {width}"
                        );
                        assert_eq!(terminal.screen().fgcolor(), fresh.screen().fgcolor());
                    }
                }
            }
        }
    }
}
