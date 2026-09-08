use crate::{Editor, Error, engine::Engine};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::{
    Event, Prompt, Role, Theme,
    actions::{Input, Request},
    system::Resource,
    terminal::Terminal,
};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::{ops::Range, os::fd::AsFd, time::Duration};

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
}
impl Interaction {
    /// Own an editor without acquiring a terminal.
    pub fn new(editor: Editor) -> Self {
        Self {
            engine: Engine::new(editor),
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            terminal: None,
        }
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
    pub fn open(
        &mut self,
        input: &impl AsFd,
        output: &impl AsFd,
        prompt: Prompt,
    ) -> Result<(), Error> {
        if self.is_open() {
            return Err(Error::State);
        }
        let (resource, _) = Resource::acquire(input, output)?;
        self.terminal = Some(Terminal::start(
            resource,
            &mut self.engine,
            prompt,
            Theme::from_environment(true),
        )?);
        Ok(())
    }
    /// Poll at most 100 ms, observing resize without owning signal policy.
    /// Incomplete sequences expire after 250 ms idle. Submit/interrupt/EOF restore
    /// the captured terminal state and release duplicated descriptors.
    pub fn poll(&mut self, timeout: Duration) -> Result<Option<Event>, Error> {
        let result = self
            .terminal
            .as_mut()
            .ok_or(Error::State)?
            .poll(&mut self.engine, timeout);
        self.reap();
        result
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
        self.terminal
            .take()
            .map_or(Ok(()), |mut t| t.close(&mut self.engine))
    }
    fn reap(&mut self) {
        if self.terminal.as_ref().is_some_and(|t| !t.active) {
            self.terminal.take();
        }
    }
}
