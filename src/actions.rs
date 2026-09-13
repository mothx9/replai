//! Public semantic interaction actions and private engine transport.
use crate::{CompletionAction, SuggestionAction};

/// Canonical editor intent, independent of terminal byte encodings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum EditAction {
    /// Move one extended grapheme left.
    Left,
    /// Move one extended grapheme right.
    Right,
    /// Move to the start of the draft.
    Home,
    /// Move to the end of the draft.
    End,
    /// Delete one extended grapheme before the cursor.
    Backspace,
    /// Delete one extended grapheme after the cursor.
    Delete,
    /// Move to the previous Unicode word boundary.
    WordLeft,
    /// Move to the next Unicode word boundary.
    WordRight,
    /// Delete the previous Unicode word.
    WordDeleteBackward,
    /// Delete the next Unicode word.
    WordDeleteForward,
    /// Recall the previous admitted history entry.
    HistoryPrevious,
    /// Recall the next admitted history entry or saved draft.
    HistoryNext,
    /// Begin or move older in reverse literal history search.
    HistorySearchOlder,
    /// Move newer in active reverse literal history search.
    HistorySearchNewer,
    /// Undo one logical edit.
    Undo,
    /// Redo one just-undone logical edit.
    Redo,
    /// Kill the previous Unicode word.
    KillWordBackward,
    /// Kill the next Unicode word.
    KillWordForward,
    /// Kill from the cursor to the logical line start.
    KillLineStart,
    /// Kill from the cursor to the logical line end.
    KillLineEnd,
    /// Insert the bounded editor kill register.
    Yank,
}

/// Semantic request that a configured key may issue. Text and paste remain literal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Action {
    /// Apply canonical editor mechanics.
    Edit(EditAction),
    /// Submit, or accept the active temporary surface according to precedence.
    Submit,
    /// Interrupt and close the interaction.
    Interrupt,
    /// Delete forward, or report end of input for an empty draft.
    DeleteOrEof,
    /// Force a complete redraw.
    Redraw,
    /// Ask the host for completion, or move next in an active menu.
    CompletionRequest,
    /// Operate explicitly on completion selection.
    Completion(CompletionAction),
    /// Accept the active reverse-search match.
    HistorySearchAccept,
    /// Dismiss reverse search without changing the draft.
    HistorySearchDismiss,
    /// Operate explicitly on the current revision-bound suggestion.
    Suggestion(SuggestionAction),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Request {
    Submit,
    Interrupt,
    Completion,
    CompletionAction(CompletionAction),
    HistorySearchAccept,
    HistorySearchDismiss,
    SuggestionAccept,
    SuggestionDismiss,
    DeleteOrEof,
    Redraw,
}

pub(crate) type EditCommand = EditAction;

#[derive(Debug, PartialEq)]
pub(crate) enum Input {
    Text(String),
    Paste(String),
    Edit(EditAction),
    Request(Request),
    Resize(usize, usize),
    TransportEof,
    Rejected(crate::EditError),
}

impl From<Action> for Input {
    fn from(action: Action) -> Self {
        match action {
            Action::Edit(a) => Self::Edit(a),
            Action::Submit => Self::Request(Request::Submit),
            Action::Interrupt => Self::Request(Request::Interrupt),
            Action::DeleteOrEof => Self::Request(Request::DeleteOrEof),
            Action::Redraw => Self::Request(Request::Redraw),
            Action::CompletionRequest => Self::Request(Request::Completion),
            Action::Completion(a) => Self::Request(Request::CompletionAction(a)),
            Action::HistorySearchAccept => Self::Request(Request::HistorySearchAccept),
            Action::HistorySearchDismiss => Self::Request(Request::HistorySearchDismiss),
            Action::Suggestion(SuggestionAction::Accept) => {
                Self::Request(Request::SuggestionAccept)
            }
            Action::Suggestion(SuggestionAction::Dismiss) => {
                Self::Request(Request::SuggestionDismiss)
            }
        }
    }
}
