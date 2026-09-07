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
        Key::Left => Input::Edit(E::Left),
        Key::Right => Input::Edit(E::Right),
        Key::Home => Input::Edit(E::Home),
        Key::End => Input::Edit(E::End),
        Key::Backspace => Input::Edit(E::Backspace),
        Key::Delete => Input::Edit(E::Delete),
        Key::Up => Input::Edit(E::HistoryPrevious),
        Key::Down => Input::Edit(E::HistoryNext),
        Key::Enter => Input::Request(R::Submit),
        Key::Interrupt => Input::Request(R::Interrupt),
        Key::Tab => Input::Request(R::Completion),
        Key::Eof => Input::Request(R::DeleteOrEof),
        Key::Clear => Input::Request(R::Redraw),
        Key::Rejected(error) => Input::Rejected(error),
        Key::IncompletePaste => return Err("incomplete bracketed paste; interaction closed"),
    })
}
