use crate::EditError;
use std::{fmt, io};

/// Recoverable edit rejection or terminal I/O failure.
#[derive(Debug)]
pub enum Error {
    /// Operation requires a different open/closed interaction state.
    State,
    /// Another interaction already owns a terminal in this process.
    Busy,
    /// Input/output are not a matching, suitably sized terminal pair.
    UnsuitableTerminal,
    /// Required terminal semantics could not be admitted before acquisition.
    CapabilityMismatch(&'static str),
    /// An invalid edit; the terminal remains active and the draft is unchanged.
    Edit(EditError),
    /// Terminal failure. Cleanup has been attempted; a cleanup failure is included in the message.
    Io(io::Error),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::State => f.write_str("invalid interaction state"),
            Self::Busy => f.write_str("another terminal interaction is active"),
            Self::UnsuitableTerminal => {
                f.write_str("unsuitable terminal descriptors or dimensions")
            }
            Self::CapabilityMismatch(reason) => f.write_str(reason),
            Self::Edit(e) => e.fmt(f),
            Self::Io(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Edit(e) => Some(e),
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<EditError> for Error {
    fn from(e: EditError) -> Self {
        Self::Edit(e)
    }
}

/// A host-visible interaction event; input is never interpreted by the library.
#[derive(Debug, PartialEq)]
pub enum Event {
    /// Enter submitted the complete input, possibly containing newlines. Terminal restored.
    Submitted(String),
    /// Ctrl-C or explicit host interruption. Terminal restored; draft remains available.
    Interrupted,
    /// Read EOF or Ctrl-D on an empty buffer. Terminal restored.
    EndOfInput,
    /// Tab requests host completion using [`crate::Interaction::editor`].
    CompletionRequested,
    /// Opt-in Enter request. The host returns a revision-bound validation result.
    /// Editing remains active; this immutable view may be retained while it continues.
    SubmissionRequested(crate::AnalysisSnapshot),
    /// Invalid input or capacity rejection; the unchanged draft remains editable.
    Rejected(EditError),
}
