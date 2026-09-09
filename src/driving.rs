//! Public scheduling vocabulary, independent of descriptors and terminal protocols.
use std::time::Instant;

/// Opaque monotonic wakeup token for one active interaction and input epoch.
///
/// Copying a token does not keep a terminal alive. A stale token or a notification
/// before its due time is a no-op. Obtain a fresh interest after every operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Deadline {
    pub(crate) at: Instant,
    pub(crate) session: u64,
    pub(crate) revision: u64,
}
impl Deadline {
    /// Monotonic due time; this has no wall-clock representation or meaning.
    pub fn at(self) -> Instant {
        self.at
    }
}

/// What can advance an active interaction. This value never waits or owns I/O.
///
/// Resize and application events are independently supplied by the host. There
/// is no periodic wake requirement when waiting for input without a deadline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitInterest {
    /// Already-read input remains. Advance without waiting for OS readiness.
    Ready,
    /// Wait for terminal input, or the deadline if present, whichever comes first.
    Input {
        /// An internal wakeup, opaque to the host beyond its monotonic due time.
        deadline: Option<Deadline>,
    },
}

/// One host-observed reason to advance the interaction, never an application action.
///
/// Hosts serialize notifications and current output/completion calls. Readiness
/// is advisory; spurious notifications produce normal no-progress, not an error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wake {
    /// Input is readable, or [`WaitInterest::Ready`] reports retained input.
    InputReady,
    /// An internal wakeup was scheduled using the supplied opaque token.
    Deadline(Deadline),
    /// Refresh resource dimensions after a host-owned resize notification.
    Resize,
}

/// Terminal features actually admitted for an open interaction.
/// Required cursor/erase semantics are validated before acquisition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InteractionFeatures {
    /// Semantic foreground/weight styling is enabled.
    pub styling: bool,
    /// The bracketed-paste terminal mode is enabled and restored by REPLAI.
    pub bracketed_paste: bool,
}

/// Result of the simple blocking tier. The terminal has been restored on success.
/// Evaluation, history admission, application exit and cancellation remain host decisions.
#[derive(Debug, PartialEq, Eq)]
pub enum ReadOutcome {
    /// Complete submitted UTF-8 input, including any pasted newlines.
    Submitted(String),
    /// Editing was interrupted; the retained draft can be inspected or cleared.
    Interrupted,
    /// Terminal EOF or Ctrl-D on an empty draft; no submission is implied.
    EndOfInput,
}
