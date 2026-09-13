//! Bounded configurable mappings from normalized physical keys to semantic actions.
use crate::{Action, EditAction};
use std::fmt;

/// Maximum number of deviations retained from the compatibility keymap.
pub const MAX_CUSTOM_BINDINGS: usize = 128;

/// Portable named keys admitted by the configurable non-modal keymap.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum NamedKey {
    /// Left arrow.
    Left,
    /// Right arrow.
    Right,
    /// Up arrow.
    Up,
    /// Down arrow.
    Down,
    /// Home.
    Home,
    /// End.
    End,
    /// Forward delete.
    Delete,
    /// Backspace.
    Backspace,
    /// Enter or return.
    Enter,
    /// Tab.
    Tab,
    /// Shift-Tab.
    BackTab,
    /// Escape.
    Escape,
    /// Page Up.
    PageUp,
    /// Page Down.
    PageDown,
}

/// A normalized configurable key. Printable text and paste are intentionally absent.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Key {
    /// A portable named key.
    Named(NamedKey),
    /// An ASCII control chord encoded as its control byte.
    Control(u8),
    /// Alt/Meta plus one printable ASCII byte.
    Meta(u8),
}
impl Key {
    /// Construct a validated ASCII control chord.
    pub fn control(byte: u8) -> Result<Self, KeyMapError> {
        (byte <= 31)
            .then_some(Self::Control(byte))
            .ok_or(KeyMapError::InvalidKey)
    }
    /// Construct a validated Alt/Meta chord.
    pub fn meta(byte: u8) -> Result<Self, KeyMapError> {
        (byte.is_ascii_graphic() || matches!(byte, 8 | 127))
            .then_some(Self::Meta(byte))
            .ok_or(KeyMapError::InvalidKey)
    }
    fn valid(self) -> bool {
        match self {
            Self::Named(_) => true,
            Self::Control(b) => b <= 31,
            Self::Meta(b) => b.is_ascii_graphic() || matches!(b, 8 | 127),
        }
    }
}

/// Invalid or over-capacity keymap configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyMapError {
    /// The chord is outside the normalized configurable domain.
    InvalidKey,
    /// The bounded override count would be exceeded.
    Limit,
}
impl fmt::Display for KeyMapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidKey => "invalid configurable key",
            Self::Limit => "custom keymap limit exceeded",
        })
    }
}
impl std::error::Error for KeyMapError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Override {
    key: Key,
    action: Option<Action>,
}

/// Compatibility defaults plus at most [`MAX_CUSTOM_BINDINGS`] sorted overrides.
/// Lookup is allocation-free and O(log n) in the bounded override count.
#[derive(Clone, Debug, Default)]
pub struct KeyMap {
    overrides: Vec<Override>,
}
impl KeyMap {
    /// Construct the qualified compatibility map with no overrides.
    pub fn new() -> Self {
        Self::default()
    }
    /// Return the effective semantic action, or `None` for an unbound chord.
    pub fn get(&self, key: Key) -> Option<Action> {
        if !key.valid() {
            return None;
        }
        match self.overrides.binary_search_by_key(&key, |entry| entry.key) {
            Ok(i) => self.overrides[i].action,
            Err(_) => compatibility(key),
        }
    }
    /// Bind or replace one mapping and return the previously effective action.
    pub fn bind(&mut self, key: Key, action: Action) -> Result<Option<Action>, KeyMapError> {
        self.set(key, Some(action))
    }
    /// Explicitly suppress a default or custom mapping.
    pub fn unbind(&mut self, key: Key) -> Result<Option<Action>, KeyMapError> {
        self.set(key, None)
    }
    /// Remove an override and restore default behavior for this chord.
    pub fn reset(&mut self, key: Key) -> Result<Option<Action>, KeyMapError> {
        if !key.valid() {
            return Err(KeyMapError::InvalidKey);
        }
        let previous = self.get(key);
        if let Ok(i) = self.overrides.binary_search_by_key(&key, |entry| entry.key) {
            self.overrides.remove(i);
        }
        Ok(previous)
    }
    /// Remove every override.
    pub fn reset_all(&mut self) {
        self.overrides.clear();
    }
    /// Number of retained overrides, including explicit unbindings.
    pub fn custom_len(&self) -> usize {
        self.overrides.len()
    }
    /// Whether any compatibility behavior has been overridden.
    pub fn is_customized(&self) -> bool {
        !self.overrides.is_empty()
    }
    fn set(&mut self, key: Key, action: Option<Action>) -> Result<Option<Action>, KeyMapError> {
        if !key.valid() {
            return Err(KeyMapError::InvalidKey);
        }
        let previous = self.get(key);
        match self.overrides.binary_search_by_key(&key, |entry| entry.key) {
            Ok(i) => self.overrides[i].action = action,
            Err(i) => {
                if self.overrides.len() == MAX_CUSTOM_BINDINGS {
                    return Err(KeyMapError::Limit);
                }
                self.overrides.insert(i, Override { key, action });
            }
        }
        Ok(previous)
    }
}

fn compatibility(key: Key) -> Option<Action> {
    use Action as A;
    use EditAction as E;
    use NamedKey as N;
    Some(match key {
        Key::Named(N::Left) => A::Edit(E::Left),
        Key::Named(N::Right) => A::Edit(E::Right),
        Key::Named(N::Home) => A::Edit(E::Home),
        Key::Named(N::End) => A::Edit(E::End),
        Key::Named(N::Backspace) => A::Edit(E::Backspace),
        Key::Named(N::Delete) => A::Edit(E::Delete),
        Key::Named(N::Up) => A::Edit(E::HistoryPrevious),
        Key::Named(N::Down) => A::Edit(E::HistoryNext),
        Key::Named(N::Enter) => A::Submit,
        Key::Named(N::Tab) => A::CompletionRequest,
        Key::Named(N::BackTab) => A::Completion(crate::CompletionAction::Previous),
        Key::Named(N::Escape) => A::Completion(crate::CompletionAction::Dismiss),
        Key::Named(N::PageUp) => A::Completion(crate::CompletionAction::PagePrevious),
        Key::Named(N::PageDown) => A::Completion(crate::CompletionAction::PageNext),
        Key::Control(3) => A::Interrupt,
        Key::Control(4) => A::DeleteOrEof,
        Key::Control(12) => A::Redraw,
        Key::Control(18) => A::Edit(E::HistorySearchOlder),
        Key::Control(19) => A::Edit(E::HistorySearchNewer),
        Key::Control(31) => A::Edit(E::Undo),
        Key::Control(23) => A::Edit(E::KillWordBackward),
        Key::Control(21) => A::Edit(E::KillLineStart),
        Key::Control(11) => A::Edit(E::KillLineEnd),
        Key::Control(25) => A::Edit(E::Yank),
        Key::Meta(b'b') => A::Edit(E::WordLeft),
        Key::Meta(b'f') => A::Edit(E::WordRight),
        Key::Meta(b'd') => A::Edit(E::WordDeleteForward),
        Key::Meta(8 | 127) => A::Edit(E::WordDeleteBackward),
        Key::Control(_) | Key::Meta(_) => return None,
    })
}

pub(crate) fn translate(
    map: &KeyMap,
    key: crate::input::Key,
) -> Result<crate::actions::Input, &'static str> {
    use crate::{actions::Input, input::Key as D};
    let key = match key {
        D::Text(text) => return Ok(Input::Text(text)),
        D::Paste(text) => return Ok(Input::Paste(text)),
        D::Rejected(error) => return Ok(Input::Rejected(error)),
        D::IncompletePaste => return Err("incomplete bracketed paste; interaction closed"),
        D::Enter => Key::Named(NamedKey::Enter),
        D::Interrupt => Key::Control(3),
        D::Eof => Key::Control(4),
        D::Home => Key::Named(NamedKey::Home),
        D::End => Key::Named(NamedKey::End),
        D::Clear => Key::Control(12),
        D::Backspace => Key::Named(NamedKey::Backspace),
        D::Delete => Key::Named(NamedKey::Delete),
        D::Tab => Key::Named(NamedKey::Tab),
        D::BackTab => Key::Named(NamedKey::BackTab),
        D::Escape => Key::Named(NamedKey::Escape),
        D::Up => Key::Named(NamedKey::Up),
        D::Down => Key::Named(NamedKey::Down),
        D::Left => Key::Named(NamedKey::Left),
        D::Right => Key::Named(NamedKey::Right),
        D::PageUp => Key::Named(NamedKey::PageUp),
        D::PageDown => Key::Named(NamedKey::PageDown),
        D::Control(byte) => Key::Control(byte),
        D::Meta(byte) => Key::Meta(byte),
    };
    Ok(map
        .get(key)
        .map(Into::into)
        .unwrap_or(Input::Rejected(crate::EditError::InvalidSequence)))
}

#[allow(dead_code)]
pub(crate) fn binding(key: crate::input::Key) -> Result<crate::actions::Input, &'static str> {
    translate(&KeyMap::new(), key)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_and_overrides_are_reversible() {
        let mut map = KeyMap::new();
        let left = Key::Named(NamedKey::Left);
        assert_eq!(map.get(left), Some(Action::Edit(EditAction::Left)));
        assert_eq!(
            map.bind(left, Action::Edit(EditAction::Redo)).unwrap(),
            Some(Action::Edit(EditAction::Left))
        );
        assert_eq!(
            map.reset(left).unwrap(),
            Some(Action::Edit(EditAction::Redo))
        );
        map.unbind(left).unwrap();
        assert_eq!(map.get(left), None);
    }
}
