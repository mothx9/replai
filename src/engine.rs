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
    completion: Option<Box<crate::completion::ActiveCompletion>>,
    analysis: Option<Box<crate::AnalysisPresentation>>,
    suggestion: Option<Box<crate::Suggestion>>,
}
pub(crate) struct Engine {
    pub editor: Editor,
    surface: Option<Surface>,
    validation: Option<Box<crate::validation::ValidationState>>,
    history_source: Option<crate::HistorySearchSource>,
    history_search: Option<Box<crate::history::HistorySearchState>>,
}
impl Engine {
    pub fn new(editor: Editor) -> Self {
        Self {
            editor,
            surface: None,
            validation: None,
            history_source: None,
            history_search: None,
        }
    }
    pub fn is_open(&self) -> bool {
        self.surface.is_some()
    }
    pub fn set_history_source(
        &mut self,
        source: Option<crate::HistorySearchSource>,
    ) -> Result<(), crate::HistoryError> {
        if self.history_search.is_some() {
            return Err(crate::HistoryError::SearchActive);
        }
        if source.as_ref().is_some_and(|s| {
            s.entries()
                .iter()
                .any(|entry| entry.len() > self.editor.capacity())
        }) {
            return Err(crate::HistoryError::EntryTooLarge);
        }
        self.history_source = source;
        Ok(())
    }
    pub fn history_search_query(&self) -> Option<&str> {
        Some(&self.history_search.as_ref()?.query)
    }
    pub fn history_search_match(&self) -> Option<&str> {
        self.history_search.as_ref()?.selected_text()
    }
    pub fn suggestion(&self) -> Option<&crate::Suggestion> {
        self.surface.as_ref()?.suggestion.as_deref()
    }
    fn begin_history_search(&mut self) {
        let limits = self
            .history_source
            .as_ref()
            .map_or_else(crate::HistorySearchLimits::default, |s| s.limits());
        let mut entries = Vec::with_capacity(limits.entries().min(64));
        let mut bytes = 0usize;
        for entry in self.editor.history_newest().chain(
            self.history_source
                .as_ref()
                .into_iter()
                .flat_map(|s| s.entries().iter().map(String::as_str)),
        ) {
            if entries.len() == limits.entries() {
                break;
            }
            let Some(total) = bytes.checked_add(entry.len()) else {
                break;
            };
            if total > limits.bytes() {
                break;
            }
            bytes = total;
            entries.push(entry.to_owned());
        }
        self.editor.break_undo_group();
        self.remove_completion();
        if let Some(validation) = &mut self.validation {
            validation.pending = None;
        }
        self.history_search = Some(Box::new(crate::history::HistorySearchState::new(
            self.editor.revision(),
            entries,
            limits.query_bytes(),
        )));
    }
    fn history_search_changed(&mut self) -> Effects {
        if let Some(surface) = &mut self.surface {
            surface.dirty = true;
            surface.damage = Damage::Rebuild;
        }
        self.flush()
    }
    fn accept_history_search(&mut self) -> Result<Effects, Error> {
        let search = self.history_search.take().expect("active search checked");
        if search.revision != self.editor.revision() {
            return Ok(self.history_search_changed());
        }
        let selected = search.selected_text().map(str::to_owned);
        let revision = self.editor.revision();
        if let Some(text) = selected {
            self.editor.replace_search_result(&text)?;
        }
        if revision != self.editor.revision() {
            self.invalidate_validation();
            self.remove_completion();
            self.remove_suggestion();
        }
        Ok(self.history_search_changed())
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
            completion: None,
            analysis: None,
            suggestion: None,
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
        if let Some(search) = &self.history_search {
            s.renderer.transition(search.frame(
                &self.editor,
                &s.prompt,
                s.size,
                s.analysis.as_deref(),
            ))
        } else if let Some(completion) = &s.completion {
            s.renderer.transition(completion.frame(
                &self.editor,
                &s.prompt,
                s.size,
                s.analysis.as_deref(),
            ))
        } else if let Some(v) = &self.validation
            && v.diagnostics.is_some()
        {
            s.renderer
                .transition(v.frame(&self.editor, &s.prompt, s.size, s.analysis.as_deref()))
        } else if let Some(suggestion) = &s.suggestion {
            let mut frame = crate::presentation::Frame::analyzed(
                &self.editor,
                &s.prompt,
                s.size.0,
                s.size.1,
                s.analysis.as_deref(),
            );
            suggestion.append(&mut frame);
            s.renderer.transition(frame)
        } else if let Some(a) = &s.analysis {
            let mut frame = crate::presentation::Frame::analyzed(
                &self.editor,
                &s.prompt,
                s.size.0,
                s.size.1,
                Some(a),
            );
            a.append_hint(&mut frame);
            s.renderer.transition(frame)
        } else {
            s.renderer
                .redraw_changed(&self.editor, &s.prompt, s.size, damage)
        }
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
        if self.history_search.is_some() {
            match &input {
                Input::Text(text) | Input::Paste(text) => {
                    if let Err(error) = self.history_search.as_mut().unwrap().insert_query(text) {
                        return Ok(self.observable(Event::Rejected(error)));
                    }
                    return Ok(self.history_search_changed());
                }
                Input::Edit(EditCommand::Backspace) => {
                    self.history_search.as_mut().unwrap().backspace();
                    return Ok(self.history_search_changed());
                }
                Input::Edit(EditCommand::KillWordBackward) => {
                    self.history_search.as_mut().unwrap().delete_query_word();
                    return Ok(self.history_search_changed());
                }
                Input::Edit(EditCommand::KillLineStart) => {
                    self.history_search.as_mut().unwrap().clear_query();
                    return Ok(self.history_search_changed());
                }
                Input::Edit(EditCommand::HistorySearchOlder) => {
                    self.history_search.as_mut().unwrap().older();
                    return Ok(self.history_search_changed());
                }
                Input::Edit(EditCommand::HistorySearchNewer) => {
                    self.history_search.as_mut().unwrap().newer();
                    return Ok(self.history_search_changed());
                }
                Input::Request(Request::Submit | Request::HistorySearchAccept) => {
                    return self.accept_history_search();
                }
                Input::Request(
                    Request::CompletionAction(crate::CompletionAction::Dismiss)
                    | Request::HistorySearchDismiss,
                ) => {
                    self.history_search = None;
                    return Ok(self.history_search_changed());
                }
                Input::Resize(..) | Input::Request(Request::Redraw) => {}
                _ => {
                    self.history_search = None;
                    if let Some(surface) = &mut self.surface {
                        surface.dirty = true;
                        surface.damage = Damage::Rebuild;
                    }
                }
            }
        }
        if self.completion_selection().is_some() {
            use crate::CompletionAction as C;
            let action = match input {
                Input::Request(Request::Completion) => Some(C::Next),
                Input::Request(Request::CompletionAction(crate::CompletionAction::Previous)) => {
                    Some(C::Previous)
                }
                Input::Request(Request::Submit) => Some(C::Accept),
                Input::Request(Request::CompletionAction(crate::CompletionAction::Dismiss)) => {
                    Some(C::Dismiss)
                }
                Input::Request(Request::CompletionAction(action)) => Some(action),
                _ => None,
            };
            if let Some(action) = action {
                return self.completion_action(action).map(|(_, effects)| effects);
            }
        }
        if self.completion_selection().is_none()
            && self.diagnostics().is_none()
            && self.suggestion().is_some()
        {
            let action = match &input {
                Input::Request(Request::SuggestionAccept) => Some(true),
                Input::Request(Request::SuggestionDismiss) => Some(false),
                Input::Edit(EditCommand::Right)
                    if self.editor.cursor() == self.editor.text().len() =>
                {
                    Some(true)
                }
                _ => None,
            };
            if let Some(accept) = action {
                return self.suggestion_action(accept).map(|(_, effects)| effects);
            }
        }
        let revision = self.editor.revision();
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
            Input::Paste(text) => {
                if let Err(error) = self.editor.insert_transaction(&text) {
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
                EditCommand::WordLeft => self.editor.word_left(),
                EditCommand::WordRight => self.editor.word_right(),
                EditCommand::WordDeleteBackward => self.editor.delete_word_backward(),
                EditCommand::WordDeleteForward => self.editor.delete_word_forward(),
                EditCommand::Undo => {
                    self.editor.undo();
                }
                EditCommand::Redo => {
                    self.editor.redo();
                }
                EditCommand::KillWordBackward => {
                    if let Err(error) = self.editor.kill_word_backward() {
                        return Ok(self.observable(Event::Rejected(error)));
                    }
                }
                EditCommand::KillWordForward => {
                    if let Err(error) = self.editor.kill_word_forward() {
                        return Ok(self.observable(Event::Rejected(error)));
                    }
                }
                EditCommand::KillLineStart => {
                    if let Err(error) = self.editor.kill_line_start() {
                        return Ok(self.observable(Event::Rejected(error)));
                    }
                }
                EditCommand::KillLineEnd => {
                    if let Err(error) = self.editor.kill_line_end() {
                        return Ok(self.observable(Event::Rejected(error)));
                    }
                }
                EditCommand::Yank => {
                    if let Err(error) = self.editor.yank() {
                        return Ok(self.observable(Event::Rejected(error)));
                    }
                }
                EditCommand::HistorySearchOlder => {
                    self.begin_history_search();
                    return Ok(self.history_search_changed());
                }
                EditCommand::HistorySearchNewer => {
                    return Ok(self.observable(Event::Rejected(crate::EditError::InvalidSequence)));
                }
                EditCommand::HistoryPrevious => {
                    if self.validation.is_none() || !self.editor.line_up() {
                        self.editor.history_up();
                    }
                }
                EditCommand::HistoryNext => {
                    if self.validation.is_none() || !self.editor.line_down() {
                        self.editor.history_down();
                    }
                }
            },
            Input::Request(Request::Submit) => {
                self.editor.break_undo_group();
                if let Some(v) = &mut self.validation {
                    v.pending = Some(self.editor.revision());
                    return Ok(self
                        .observable(Event::SubmissionRequested(self.editor.analysis_snapshot())));
                }
                return Ok(self.finish(Event::Submitted(self.editor.text().into())));
            }
            Input::Request(Request::Interrupt) => return Ok(self.finish(Event::Interrupted)),
            Input::TransportEof => return Ok(self.finish(Event::EndOfInput)),
            Input::Request(Request::DeleteOrEof) if self.editor.text().is_empty() => {
                return Ok(self.finish(Event::EndOfInput));
            }
            Input::Request(Request::DeleteOrEof) => self.editor.delete(),
            Input::Request(Request::Completion) => {
                self.editor.break_undo_group();
                if let Some(spaces) = self.continuation_indent() {
                    if let Err(error) = self.editor.insert_transaction(&"    "[..spaces]) {
                        return Ok(self.observable(Event::Rejected(error)));
                    }
                } else {
                    return Ok(self.observable(Event::CompletionRequested));
                }
            }
            Input::Request(
                Request::CompletionAction(crate::CompletionAction::Previous)
                | Request::CompletionAction(crate::CompletionAction::Dismiss),
            ) => {
                if matches!(
                    input,
                    Input::Request(Request::CompletionAction(crate::CompletionAction::Dismiss))
                ) && self.clear_diagnostics()
                {
                    return Ok(self.flush());
                }
                // Preserve compatibility rejection outside a completion surface.
                return Ok(self.observable(Event::Rejected(crate::EditError::InvalidSequence)));
            }
            Input::Request(
                Request::CompletionAction(_)
                | Request::HistorySearchAccept
                | Request::HistorySearchDismiss
                | Request::SuggestionAccept
                | Request::SuggestionDismiss,
            ) => {
                return Ok(self.observable(Event::Rejected(crate::EditError::InvalidSequence)));
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
        if revision != self.editor.revision() {
            self.history_search = None;
            self.remove_suggestion();
            if self.analysis_presentation().is_some() {
                damage = Damage::Rebuild;
            }
            self.invalidate_validation();
        }
        let s = self.surface.as_mut().unwrap();
        if revision != self.editor.revision() && s.completion.take().is_some() {
            damage = Damage::Rebuild;
        }
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
        self.history_search = None;
        self.remove_completion();
        self.invalidate_validation();
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
        self.history_search = None;
        self.editor.break_undo_group();
        self.remove_completion();
        self.invalidate_validation();
        let mut effects = self.flush();
        if let Some(mut s) = self.surface.take() {
            effects.mutations.extend(s.renderer.leave(false));
        }
        effects
    }
    pub fn abandon(&mut self) {
        self.history_search = None;
        self.invalidate_validation();
        self.surface = None;
    }
    pub fn complete(&mut self, range: Range<usize>, text: &str) -> Result<Effects, Error> {
        if !self.is_open() {
            return Err(Error::State);
        }
        let revision = self.editor.revision();
        self.editor.replace(range, text)?;
        if revision != self.editor.revision() {
            self.invalidate_validation();
            self.remove_completion();
            self.remove_suggestion();
        }
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
    pub fn completion_selection(&self) -> Option<crate::CompletionSelection> {
        self.surface
            .as_ref()?
            .completion
            .as_ref()
            .map(|c| c.selection())
    }
    fn remove_completion(&mut self) {
        if let Some(s) = &mut self.surface
            && s.completion.take().is_some()
        {
            s.dirty = true;
            s.damage = Damage::Rebuild;
        }
    }
    fn remove_suggestion(&mut self) {
        if let Some(s) = &mut self.surface
            && s.suggestion.take().is_some()
        {
            s.dirty = true;
            s.damage = Damage::Rebuild;
        }
    }
    pub fn present_suggestion(
        &mut self,
        suggestion: crate::Suggestion,
    ) -> Result<(crate::AnalysisOutcome, Effects), crate::SuggestionError> {
        if suggestion.revision() != self.editor.revision() {
            return Ok((crate::AnalysisOutcome::Stale, Effects::default()));
        }
        if !self.is_open() {
            return Err(Error::State.into());
        }
        suggestion.validate(&self.editor)?;
        let s = self.surface.as_mut().unwrap();
        s.suggestion = Some(Box::new(suggestion));
        s.dirty = true;
        s.damage = Damage::Rebuild;
        Ok((crate::AnalysisOutcome::Applied, self.flush()))
    }
    pub fn suggestion_action(
        &mut self,
        accept: bool,
    ) -> Result<(crate::AnalysisOutcome, Effects), Error> {
        let suggestion = self.suggestion().ok_or(Error::State)?;
        if suggestion.revision() != self.editor.revision() {
            return Ok((crate::AnalysisOutcome::Stale, Effects::default()));
        }
        if accept
            && self
                .validation
                .as_ref()
                .is_some_and(|v| v.diagnostics.is_some())
        {
            return Err(Error::State);
        }
        if accept {
            let text = suggestion.text().to_owned();
            self.editor.insert_transaction(&text)?;
            self.remove_suggestion();
            self.invalidate_validation();
            self.remove_completion();
        } else {
            self.remove_suggestion();
        }
        let s = self.surface.as_mut().ok_or(Error::State)?;
        s.dirty = true;
        s.damage = Damage::Rebuild;
        Ok((crate::AnalysisOutcome::Applied, self.flush()))
    }
    pub fn present_completions(
        &mut self,
        set: crate::CompletionSet,
    ) -> Result<(crate::AnalysisOutcome, Effects), crate::CompletionError> {
        if !self.is_open() {
            return Err(Error::State.into());
        }
        if set.revision() != self.editor.revision() {
            return Ok((crate::AnalysisOutcome::Stale, Effects::default()));
        }
        if self.history_search.is_some() {
            return Err(Error::State.into());
        }
        set.validate(&self.editor)?;
        self.clear_diagnostics();
        self.remove_completion();
        if !set.candidates().is_empty() {
            let s = self.surface.as_mut().unwrap();
            s.completion = Some(Box::new(crate::completion::ActiveCompletion {
                set,
                selected: 0,
            }));
            s.dirty = true;
            s.damage = Damage::Rebuild;
        }
        Ok((crate::AnalysisOutcome::Applied, self.flush()))
    }
    pub fn completion_action(
        &mut self,
        action: crate::CompletionAction,
    ) -> Result<(crate::AnalysisOutcome, Effects), Error> {
        use crate::{AnalysisOutcome as A, CompletionAction as C};
        let s = self.surface.as_mut().ok_or(Error::State)?;
        let c = s.completion.as_mut().ok_or(Error::State)?;
        if c.set.revision() != self.editor.revision() {
            // Defensive invariant: every public mutation already dismisses.
            return Ok((A::Stale, Effects::default()));
        }
        match action {
            C::Next | C::Previous | C::PageNext | C::PagePrevious | C::First | C::Last => {
                let page = c.page_size(s.size.1);
                c.navigate(action, page);
            }
            C::Accept => {
                let candidate = c.candidate();
                let revision = self.editor.revision();
                self.editor.replace_at(
                    c.set.revision(),
                    candidate.range(),
                    candidate.replacement(),
                )?;
                if self.editor.revision() != revision {
                    self.invalidate_validation();
                    self.remove_suggestion();
                }
                self.remove_completion();
            }
            C::Dismiss => self.remove_completion(),
        }
        let s = self.surface.as_mut().unwrap();
        s.dirty = true;
        s.damage = Damage::Rebuild;
        Ok((A::Applied, self.flush()))
    }
    pub fn analysis_presentation(&self) -> Option<&crate::AnalysisPresentation> {
        self.surface.as_ref()?.analysis.as_deref()
    }
    fn remove_analysis(&mut self) {
        if let Some(s) = &mut self.surface
            && s.analysis.take().is_some()
        {
            s.dirty = true;
            s.damage = Damage::Rebuild;
        }
    }
    pub fn present_analysis(
        &mut self,
        result: crate::AnalysisPresentation,
    ) -> Result<(crate::AnalysisOutcome, Effects), crate::AnalysisPresentationError> {
        if result.revision() != self.editor.revision() {
            return Ok((crate::AnalysisOutcome::Stale, Effects::default()));
        }
        if !self.is_open() {
            return Err(Error::State.into());
        }
        result.validate(self.editor.text())?;
        let s = self.surface.as_mut().unwrap();
        s.analysis = if result.is_empty() {
            None
        } else {
            Some(Box::new(result))
        };
        s.dirty = true;
        s.damage = Damage::Rebuild;
        Ok((crate::AnalysisOutcome::Applied, self.flush()))
    }
    // Fixed validated-multiline binding, not language-aware indentation. Inspect
    // only the current logical prefix; menu navigation is handled before editing.
    fn continuation_indent(&self) -> Option<usize> {
        self.validation.as_ref()?;
        let before = &self.editor.text()[..self.editor.cursor()];
        let (_, prefix) = before.rsplit_once('\n')?;
        let mut remainder = 0;
        for byte in prefix.bytes() {
            match byte {
                b' ' => remainder = (remainder + 1) % 4,
                b'\t' => remainder = 0,
                _ => return None,
            }
        }
        Some(4 - remainder)
    }
    pub fn submission_policy(&self) -> crate::SubmissionPolicy {
        if self.validation.is_some() {
            crate::SubmissionPolicy::Validated
        } else {
            crate::SubmissionPolicy::Direct
        }
    }
    pub fn set_submission_policy(&mut self, policy: crate::SubmissionPolicy) -> Result<(), Error> {
        if self.is_open() {
            return Err(Error::State);
        }
        self.validation = match policy {
            crate::SubmissionPolicy::Direct => None,
            crate::SubmissionPolicy::Validated => Some(Box::default()),
        };
        Ok(())
    }
    pub fn diagnostics(&self) -> Option<&[crate::Diagnostic]> {
        self.validation.as_ref()?.diagnostics.as_deref()
    }
    fn clear_diagnostics(&mut self) -> bool {
        let removed = self
            .validation
            .as_mut()
            .is_some_and(|v| v.diagnostics.take().is_some());
        if removed && let Some(s) = &mut self.surface {
            s.dirty = true;
            s.damage = Damage::Rebuild;
        }
        removed
    }
    fn invalidate_validation(&mut self) {
        self.remove_analysis();
        self.clear_diagnostics();
        if let Some(v) = &mut self.validation {
            v.pending = None;
        }
    }
    pub fn apply_validation(
        &mut self,
        result: crate::ValidationResult,
    ) -> Result<(crate::AnalysisOutcome, Effects), crate::ValidationError> {
        use crate::{AnalysisOutcome as A, ValidationDisposition as V, ValidationError as E};
        // Stale provenance wins even after a submission closed the terminal.
        if result.revision() != self.editor.revision() {
            return Ok((A::Stale, Effects::default()));
        }
        if !self.is_open() {
            return Err(Error::State.into());
        }
        if self.validation.as_ref().and_then(|v| v.pending) != Some(result.revision()) {
            return Err(E::NoRequest);
        }
        if let V::Invalid(diagnostics) = result.disposition() {
            for d in diagnostics {
                if let Some(range) = d.range() {
                    self.editor.validate_replacement(&range, "")?;
                }
            }
        }
        let effects = match result.into_disposition() {
            V::Complete => self.finish(Event::Submitted(self.editor.text().into())),
            V::Incomplete => {
                self.editor.insert_transaction("\n")?;
                self.invalidate_validation();
                self.remove_suggestion();
                self.remove_completion();
                let s = self.surface.as_mut().unwrap();
                s.dirty = true;
                s.damage = Damage::Rebuild;
                self.flush()
            }
            V::Invalid(diagnostics) => {
                self.remove_completion();
                let v = self.validation.as_mut().unwrap();
                v.pending = None;
                v.diagnostics = Some(diagnostics);
                let s = self.surface.as_mut().unwrap();
                s.dirty = true;
                s.damage = Damage::Rebuild;
                self.flush()
            }
        };
        Ok((A::Applied, effects))
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

#[cfg(test)]
#[path = "completion_tests.rs"]
mod completion_tests;

#[cfg(test)]
#[path = "validation_tests.rs"]
mod validation_tests;

#[cfg(test)]
#[path = "analysis_presentation_tests.rs"]
mod analysis_presentation_tests;

#[cfg(test)]
#[path = "ergonomics_tests.rs"]
mod ergonomics_tests;

#[cfg(test)]
#[path = "suggestion_tests.rs"]
mod suggestion_tests;
