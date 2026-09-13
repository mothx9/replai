//! Compatibility bindings, separate from protocol recognition and editor storage.
use crate::{
    actions::{EditCommand, Input, Request},
    input::Key,
};

// Current compatibility keymap. A future structured-key source can use this
// action vocabulary without emulating Unix control bytes or editing storage.
pub(crate) fn binding(key: Key) -> Result<Input, &'static str> {
    use EditCommand as E;
    use Request as R;
    Ok(match key {
        Key::Text(text) => Input::Text(text),
        Key::Paste(text) => Input::Paste(text),
        Key::Left => Input::Edit(E::Left),
        Key::Right => Input::Edit(E::Right),
        Key::Meta(b'b') => Input::Edit(E::WordLeft),
        Key::Meta(b'f') => Input::Edit(E::WordRight),
        Key::Meta(b'd') => Input::Edit(E::WordDeleteForward),
        Key::Meta(8 | 127) => Input::Edit(E::WordDeleteBackward),
        Key::Control(18) => Input::Edit(E::HistorySearchOlder),
        Key::Control(19) => Input::Edit(E::HistorySearchNewer),
        Key::Control(31) => Input::Edit(E::Undo),
        Key::Control(23) => Input::Edit(E::KillWordBackward),
        Key::Control(21) => Input::Edit(E::KillLineStart),
        Key::Control(11) => Input::Edit(E::KillLineEnd),
        Key::Control(25) => Input::Edit(E::Yank),
        Key::Control(_) | Key::Meta(_) => Input::Rejected(crate::EditError::InvalidSequence),
        Key::Home => Input::Edit(E::Home),
        Key::End => Input::Edit(E::End),
        Key::Backspace => Input::Edit(E::Backspace),
        Key::Delete => Input::Edit(E::Delete),
        Key::Up => Input::Edit(E::HistoryPrevious),
        Key::Down => Input::Edit(E::HistoryNext),
        Key::Enter => Input::Request(R::Submit),
        Key::Interrupt => Input::Request(R::Interrupt),
        Key::Tab => Input::Request(R::Completion),
        Key::BackTab => Input::Request(R::CompletionPrevious),
        Key::Escape => Input::Request(R::DismissCompletion),
        Key::Eof => Input::Request(R::DeleteOrEof),
        Key::Clear => Input::Request(R::Redraw),
        Key::Rejected(error) => Input::Rejected(error),
        Key::IncompletePaste => return Err("incomplete bracketed paste; interaction closed"),
    })
}
