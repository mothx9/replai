//! Normalized input, independent of physical keys, byte protocols and resources.
use crate::EditError;

#[derive(Debug, PartialEq)]
pub(crate) enum EditCommand {
    Left,
    Right,
    Home,
    End,
    Backspace,
    Delete,
    HistoryPrevious,
    HistoryNext,
}
#[derive(Debug, PartialEq)]
pub(crate) enum Request {
    Submit,
    Interrupt,
    Completion,
    CompletionPrevious,
    DismissCompletion,
    DeleteOrEof,
    Redraw,
}
#[derive(Debug, PartialEq)]
pub(crate) enum Input {
    Text(String),
    Edit(EditCommand),
    Request(Request),
    Resize(usize, usize),
    TransportEof,
    Rejected(EditError),
}
