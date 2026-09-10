use crate::{AnalysisSnapshot, DraftRevision, Editor, Error, engine::Engine};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::{
    Event, Prompt, Role, Theme,
    actions::{Input, Request},
    system::Resource,
    terminal::Terminal,
};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::{collections::VecDeque, ops::Range, os::fd::AsFd, time::Duration};

/// Host-owned interaction state with a scoped system-terminal compatibility façade.
///
/// Editor, events and the internal interaction engine are platform-neutral.
/// System acquisition/polling is available on Linux and macOS.
/// Moving this value is safe. No signals or background threads are installed.
/// The current backend admits one active terminal per process; this restriction
/// belongs to resource acquisition, not to the deterministic engine.
pub struct Interaction {
    engine: Engine,
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    terminal: Option<Terminal<Resource>>,
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pending: VecDeque<u8>,
}
impl Interaction {
    /// Own an editor without acquiring a terminal.
    pub fn new(editor: Editor) -> Self {
        Self {
            engine: Engine::new(editor),
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            terminal: None,
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            pending: VecDeque::new(),
        }
    }
    /// Configure opt-in host validation while closed. Direct remains the default.
    /// The policy persists across reopen; pending requests and diagnostics do not.
    pub fn set_submission_policy(&mut self, policy: crate::SubmissionPolicy) -> Result<(), Error> {
        if self.is_open() {
            return Err(Error::State);
        }
        self.engine.set_submission_policy(policy)
    }
    /// Current Enter/vertical-navigation policy; no terminal acquisition required.
    pub fn submission_policy(&self) -> crate::SubmissionPolicy {
        self.engine.submission_policy()
    }
    /// Complete retained diagnostic messages for the current invalid draft, including
    /// explanations omitted/ellipsized by the bounded temporary display.
    pub fn diagnostics(&self) -> Option<&[crate::Diagnostic]> {
        self.engine.diagnostics()
    }
    /// Current draft identity within the retained editor, including while closed.
    pub fn revision(&self) -> DraftRevision {
        self.engine.editor.revision()
    }
    /// Retain one immutable view while this interaction continues to advance.
    pub fn analysis_snapshot(&self) -> AnalysisSnapshot {
        self.engine.editor.analysis_snapshot()
    }
    /// Inspect draft text and byte cursor in any lifecycle state.
    pub fn editor(&self) -> &Editor {
        &self.engine.editor
    }
    /// Mutate editing/history state while closed; active edits use completion.
    pub fn editor_mut(&mut self) -> Result<&mut Editor, Error> {
        if self.is_open() {
            Err(Error::State)
        } else {
            Ok(&mut self.engine.editor)
        }
    }
    /// Whether an interaction is active or a system resource still awaits cleanup.
    pub fn is_open(&self) -> bool {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        if self.terminal.is_some() {
            return true;
        }
        self.engine.is_open()
    }
}
#[cfg(any(target_os = "linux", target_os = "macos"))]
impl Interaction {
    /// Acquire a matching POSIX TTY pair and draw the retained draft.
    /// Caller descriptors are duplicated and remain owned by the caller.
    /// Failure preserves the editor; closing permits reopening with another prompt.
    /// This compatibility entry assumes VT cursor/erase/paste even for TERM=dumb
    /// (which disables styling). New hosts can use explicit admission instead.
    pub fn open(
        &mut self,
        input: &impl AsFd,
        output: &impl AsFd,
        prompt: Prompt,
    ) -> Result<(), Error> {
        self.open_with_theme(input, output, prompt, Theme::from_environment(true))
    }
    /// Acquire an interaction with an explicitly resolved role theme. Callers
    /// supplying terminal facts retain responsibility for their color policy.
    pub fn open_with_theme(
        &mut self,
        input: &impl AsFd,
        output: &impl AsFd,
        prompt: Prompt,
        theme: Theme,
    ) -> Result<(), Error> {
        self.open_admitted(
            input,
            output,
            prompt,
            crate::TerminalConfig::compatibility(theme),
            crate::InteractionRequirements::Editing,
        )
    }
    /// Read one editable line on stdin/stdout without a host polling loop.
    /// Uses conservative environment admission, the same retained editor/history,
    /// and the compatibility waiter. No history is admitted automatically. Tab
    /// is declined; recoverable input rejection is reported as safe feedback.
    /// On success the terminal is closed. Errors attempt cleanup; explicit close
    /// can retry a retained failed restoration. No application signal policy is installed.
    pub fn read_line(&mut self, prompt: Prompt) -> Result<crate::ReadOutcome, Error> {
        if self.submission_policy() != crate::SubmissionPolicy::Direct {
            return Err(Error::State);
        }
        self.open_admitted(
            &std::io::stdin(),
            &std::io::stdout(),
            prompt,
            crate::TerminalConfig::from_environment(),
            crate::InteractionRequirements::Editing,
        )?;
        loop {
            match self.poll(Duration::from_millis(100))? {
                Some(Event::Submitted(text)) => return Ok(crate::ReadOutcome::Submitted(text)),
                Some(Event::Interrupted) => return Ok(crate::ReadOutcome::Interrupted),
                Some(Event::EndOfInput) => return Ok(crate::ReadOutcome::EndOfInput),
                Some(Event::Rejected(error)) => {
                    self.external_output(Role::Warning, &error.to_string())?
                }
                Some(Event::CompletionRequested) | None => {}
                Some(Event::SubmissionRequested(_)) => {
                    unreachable!("blocking entry requires direct submission")
                }
            }
        }
    }
    /// Open using conservative terminal facts from the environment. This does
    /// not wait or select a scheduler. Missing/dumb TERM refuses before raw mode;
    /// hosts with independent capability evidence can use [`Self::open_with_config`].
    pub fn open_driven(
        &mut self,
        input: &impl AsFd,
        output: &impl AsFd,
        prompt: Prompt,
    ) -> Result<(), Error> {
        self.open_admitted(
            input,
            output,
            prompt,
            crate::TerminalConfig::from_environment(),
            crate::InteractionRequirements::Driven,
        )
    }
    /// Admit required mechanics and optional styling/paste before raw mode.
    /// Native resource observations take precedence over configured assumptions.
    /// No signals, threads, reactor, or periodic timer are installed.
    pub fn open_with_config(
        &mut self,
        input: &impl AsFd,
        output: &impl AsFd,
        prompt: Prompt,
        config: crate::TerminalConfig,
    ) -> Result<(), Error> {
        self.open_admitted(
            input,
            output,
            prompt,
            config,
            crate::InteractionRequirements::Driven,
        )
    }
    fn open_admitted(
        &mut self,
        input: &impl AsFd,
        output: &impl AsFd,
        prompt: Prompt,
        config: crate::TerminalConfig,
        requirements: crate::InteractionRequirements,
    ) -> Result<(), Error> {
        if self.is_open() {
            return Err(Error::State);
        }
        let (resource, theme, capabilities) =
            Resource::acquire(input, output, config, requirements)?;
        let mut terminal =
            Terminal::start_admitted(resource, &mut self.engine, prompt, theme, capabilities)?;
        terminal.pending = std::mem::take(&mut self.pending);
        self.terminal = Some(terminal);
        Ok(())
    }
    /// Inspect admitted protocol assumptions, native observations, requirements and
    /// degradation. No I/O or environment access. Dimensions reflect the last
    /// successful acquisition/refresh, not an unsolicited size query. Closing
    /// invalidates this accessor; a copied snapshot carries no terminal ownership.
    pub fn capabilities(&self) -> Result<crate::TerminalCapabilities, Error> {
        self.terminal
            .as_ref()
            .filter(|t| t.active && self.engine.is_open())
            .and_then(|t| t.capabilities)
            .ok_or(Error::State)
    }
    /// Current required wakeup, with no I/O, size query, allocation or wait.
    /// Re-query after every advancement or host mutation. Idle input without a
    /// deadline has no periodic REPLAI wake; resize must be notified by the host.
    pub fn wait_interest(&self) -> Result<crate::WaitInterest, Error> {
        let terminal = self
            .terminal
            .as_ref()
            .filter(|t| t.active && self.engine.is_open())
            .ok_or(Error::State)?;
        Ok(terminal.interest())
    }
    /// Advance one host notification. Input advancement only checks immediately
    /// ready input; it never blocks waiting for another key. Already-read bytes
    /// stop at semantic events and remain retained, including across reopen.
    /// A stale/early deadline is a no-op; resize queries this resource's geometry.
    /// Hosts serialize this with completion/output/interrupt; this is not an
    /// async-signal-safe handler or a concurrent-writer interface.
    pub fn advance(&mut self, wake: crate::Wake) -> Result<Option<Event>, Error> {
        if !self.engine.is_open() {
            return Err(Error::State);
        }
        let result = self
            .terminal
            .as_mut()
            .ok_or(Error::State)?
            .advance(&mut self.engine, wake);
        self.reap();
        result
    }
    /// Borrow the active REPLAI-owned POSIX readiness source for a host reactor.
    /// The borrow cannot outlive close or transfer restoration ownership. REPLAI
    /// remains the sole reader: do not read from this source or change its flags.
    /// Register for readability, release the borrow, then call [`Self::advance`].
    ///
    /// ```compile_fail
    /// # use replai::Interaction;
    /// # fn wrong(interaction: &mut Interaction) {
    /// let source = interaction.input_source().unwrap();
    /// interaction.close().unwrap(); // cannot close while the borrow is still used
    /// drop(source);
    /// # }
    /// ```
    /// Unregister any raw reactor registration before closing/reopening; a raw
    /// descriptor copied into a reactor does not extend this borrow's lifetime.
    pub fn input_source(&self) -> Result<std::os::fd::BorrowedFd<'_>, Error> {
        let terminal = self
            .terminal
            .as_ref()
            .filter(|t| t.active && self.engine.is_open())
            .ok_or(Error::State)?;
        Ok(terminal.resource.input_source())
    }
    /// Features admitted at acquisition; optional losses are visible here.
    pub fn features(&self) -> Result<crate::InteractionFeatures, Error> {
        let terminal = self
            .terminal
            .as_ref()
            .filter(|t| t.active && self.engine.is_open())
            .ok_or(Error::State)?;
        Ok(terminal.features)
    }
    /// Poll at most 100 ms, observing resize without owning signal policy.
    /// Incomplete sequences expire after 250 ms idle. Submit/interrupt/EOF restore
    /// the captured terminal state and release duplicated descriptors.
    /// Already-ready input is read in bounded chunks. Read-ahead after a returned
    /// event remains owned by this Interaction, including across close/reopen.
    pub fn poll(&mut self, timeout: Duration) -> Result<Option<Event>, Error> {
        let result = self
            .terminal
            .as_mut()
            .ok_or(Error::State)?
            .poll(&mut self.engine, timeout);
        self.reap();
        result
    }
    /// Apply existing completion replacement only to its originating current draft.
    ///
    /// Calls are serialized by exclusive ownership. Stale results return without
    /// validation, rendering or terminal I/O. A closed interaction returns State.
    /// Current malformed replacements return Edit errors. No host analysis runs here.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub fn complete_at(
        &mut self,
        revision: DraftRevision,
        range: Range<usize>,
        replacement: &str,
    ) -> Result<crate::AnalysisOutcome, Error> {
        let terminal = self.terminal.as_mut().ok_or(Error::State)?;
        let (outcome, effects) = self.engine.complete_at(revision, range, replacement)?;
        if outcome == crate::AnalysisOutcome::Stale {
            return Ok(outcome);
        }
        let result = terminal.apply(&mut self.engine, effects).map(|_| outcome);
        self.reap();
        result
    }

    /// Install host-ordered candidates for their originating draft revision.
    /// Stale results produce no terminal bytes or state changes. All candidates
    /// validate before activation. Empty sets dismiss; even one item needs Enter.
    /// Same-revision deliveries replace in delivery order; host job preference
    /// and context freshness remain host-owned. Failure uses normal cleanup.
    pub fn present_completions(
        &mut self,
        set: crate::CompletionSet,
    ) -> Result<crate::AnalysisOutcome, crate::CompletionError> {
        let terminal = self.terminal.as_mut().ok_or(Error::State)?;
        let (outcome, effects) = self.engine.present_completions(set)?;
        if outcome == crate::AnalysisOutcome::Stale {
            return Ok(outcome);
        }
        let result = terminal.apply(&mut self.engine, effects).map(|_| outcome);
        self.reap();
        result.map_err(Into::into)
    }
    /// Inspect temporary selection without borrowing candidate storage or a resource.
    pub fn completion_selection(&self) -> Option<crate::CompletionSelection> {
        self.engine.completion_selection()
    }
    /// Navigate, accept or dismiss an active menu through exclusive ownership.
    /// No active menu is a State error. Navigation/dismissal leave revision intact.
    /// Acceptance validates and replaces once; I0 no-op rules still apply.
    /// Tab/Shift-Tab, Enter and Escape expose these same operations to terminal input.
    pub fn completion_action(
        &mut self,
        action: crate::CompletionAction,
    ) -> Result<crate::AnalysisOutcome, Error> {
        let terminal = self.terminal.as_mut().ok_or(Error::State)?;
        let (outcome, effects) = self.engine.completion_action(action)?;
        if outcome == crate::AnalysisOutcome::Stale {
            return Ok(outcome);
        }
        let result = terminal.apply(&mut self.engine, effects).map(|_| outcome);
        self.reap();
        result
    }
    /// Deliver one decision for a pending Enter request. Exclusive ownership makes
    /// revision check, range validation and mutation/submission one logical operation.
    /// Stale results perform no I/O, including after terminal restoration. Complete
    /// returns Submitted here exactly once, not again through poll/advance.
    /// Malformed current data leaves the pending request and draft unchanged.
    pub fn apply_validation(
        &mut self,
        result: crate::ValidationResult,
    ) -> Result<crate::ValidationOutcome, crate::ValidationError> {
        let (analysis, effects) = self.engine.apply_validation(result)?;
        if analysis == crate::AnalysisOutcome::Stale {
            return Ok(crate::ValidationOutcome {
                analysis,
                event: None,
            });
        }
        let event = self
            .terminal
            .as_mut()
            .ok_or(Error::State)?
            .apply(&mut self.engine, effects);
        self.reap();
        Ok(crate::ValidationOutcome {
            analysis,
            event: event?,
        })
    }
    /// Apply a host-selected grapheme-aligned replacement and redraw.
    /// Invalid edits preserve text/cursor and leave the interaction active.
    pub fn complete(&mut self, range: Range<usize>, replacement: &str) -> Result<(), Error> {
        let terminal = self.terminal.as_mut().ok_or(Error::State)?;
        let effects = self.engine.complete(range, replacement)?;
        let result = terminal.apply(&mut self.engine, effects).map(|_| ());
        self.reap();
        result
    }
    /// Write safe host text as a line transaction and restore draft/cursor.
    /// LF/TAB and CRLF are admitted; other controls reject before mutation.
    /// Hosts serialize writes; raw input and queued bytes are retained.
    pub fn external_output(&mut self, role: Role, text: &str) -> Result<(), Error> {
        let terminal = self.terminal.as_mut().ok_or(Error::State)?;
        let effects = self.engine.external_output(role, text)?;
        let result = terminal.apply(&mut self.engine, effects).map(|_| ());
        self.reap();
        result
    }
    /// Present a bounded semantic document at the active terminal width, then
    /// restore the exact draft/cursor. Rejection is atomic; I/O failure cleans up.
    pub fn output_document(&mut self, document: &crate::Document) -> Result<(), Error> {
        let terminal = self.terminal.as_mut().ok_or(Error::State)?;
        let effects = self.engine.output_document(document)?;
        let result = terminal.apply(&mut self.engine, effects).map(|_| ());
        self.reap();
        result
    }
    /// Deliver a host-observed editing interrupt and release the terminal.
    /// This ordinary method is not an async-signal-safe handler.
    pub fn interrupt(&mut self) -> Result<Event, Error> {
        let terminal = self.terminal.as_mut().ok_or(Error::State)?;
        let effects = self.engine.apply(Input::Request(Request::Interrupt))?;
        let result = terminal
            .apply(&mut self.engine, effects)
            .map(|event| event.unwrap());
        self.reap();
        result
    }
    /// Restore and release the terminal, retaining editor/history. Idempotent.
    /// Drop also attempts restoration without panicking; explicit close reports errors.
    pub fn close(&mut self) -> Result<(), Error> {
        if let Some(mut t) = self.terminal.take() {
            let result = t.close(&mut self.engine);
            self.pending = std::mem::take(&mut t.pending);
            result
        } else {
            Ok(())
        }
    }
    fn reap(&mut self) {
        if self.terminal.as_ref().is_some_and(|t| !t.active) {
            self.pending = std::mem::take(&mut self.terminal.take().unwrap().pending);
        }
    }
}
