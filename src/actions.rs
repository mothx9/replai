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
    WordLeft,
    WordRight,
    WordDeleteBackward,
    WordDeleteForward,
    HistorySearchOlder,
    HistorySearchNewer,
    Undo,
    #[allow(dead_code)] // Public Editor API; intentionally unbound until configurable keymaps.
    Redo,
    KillWordBackward,
    #[allow(dead_code)] // Public Editor API; intentionally unbound until configurable keymaps.
    KillWordForward,
    KillLineStart,
    KillLineEnd,
    Yank,
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
    Paste(String),
    Edit(EditCommand),
    Request(Request),
    Resize(usize, usize),
    TransportEof,
    Rejected(EditError),
}
