//! Revision-bound acceptable ghost text, separate from informational analysis hints.
use crate::{AnalysisSnapshot, DraftRevision, EditError, Error};
use std::fmt;

/// Maximum bytes in the single active suggestion.
pub const MAX_SUGGESTION_BYTES: usize = 4_096;
/// Maximum source values inspected by a generic suggestion helper.
pub const MAX_SUGGESTION_SOURCE_ITEMS: usize = 4_096;
/// Maximum total source bytes inspected by a generic suggestion helper.
pub const MAX_SUGGESTION_SOURCE_BYTES: usize = 4 * 1024 * 1024;

/// Explicit operation over the one active suggestion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuggestionAction {
    /// Insert the suffix as one canonical undoable edit.
    Accept,
    /// Remove the suffix without changing draft or revision.
    Dismiss,
}

/// One immutable suffix for an exact draft-and-cursor revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Suggestion {
    revision: DraftRevision,
    text: Box<str>,
}
impl Suggestion {
    /// Construct nonempty bounded editor text. Delivery additionally requires an
    /// end-of-draft cursor and enough remaining editor capacity.
    pub fn new(revision: DraftRevision, text: &str) -> Result<Self, SuggestionError> {
        if text.is_empty() || text.len() > MAX_SUGGESTION_BYTES {
            return Err(SuggestionError::Limit);
        }
        if !crate::core::valid_text(text) {
            return Err(SuggestionError::Edit(EditError::InvalidText));
        }
        Ok(Self {
            revision,
            text: text.into(),
        })
    }
    /// Originating canonical draft-and-cursor identity.
    pub fn revision(&self) -> DraftRevision {
        self.revision
    }
    /// Exact suffix inserted only by explicit acceptance.
    pub fn text(&self) -> &str {
        &self.text
    }
    pub(crate) fn validate(&self, editor: &crate::Editor) -> Result<(), SuggestionError> {
        if editor.cursor() != editor.text().len() {
            return Err(SuggestionError::NotAtEnd);
        }
        if self.text.len() > editor.capacity().saturating_sub(editor.text().len()) {
            return Err(SuggestionError::Edit(EditError::Capacity));
        }
        Ok(())
    }
    pub(crate) fn append(&self, frame: &mut crate::presentation::Frame) {
        use crate::{Role, presentation::Run};
        if frame.source_cursor != frame.source_len {
            return;
        }
        let available = frame.columns.saturating_sub(frame.end.col + 1);
        if available < 5 {
            return;
        }
        let visible = self.text.replace('\n', "↵").replace('\t', "⇥");
        let clipped = crate::completion::clip(&visible, available - 4);
        let text = format!(" [→{clipped}]");
        let width = crate::width::cells(&text);
        frame.lines.last_mut().unwrap().0.extend([
            Run::Style(Role::Dim),
            Run::Text(text),
            Run::Style(Role::Default),
        ]);
        *frame.widths.last_mut().unwrap() += width;
        frame.end.col += width;
    }
}

/// Invalid suggestion data, placement, or active interaction failure.
#[derive(Debug)]
pub enum SuggestionError {
    /// Empty, oversized, or over-budget helper input.
    Limit,
    /// Suggestions are suffixes and therefore require an end-of-draft cursor.
    NotAtEnd,
    /// Suggestion insertion text is invalid or exceeds editor capacity.
    Edit(EditError),
    /// Lifecycle or terminal failure under the normal cleanup contract.
    Interaction(Error),
}
impl fmt::Display for SuggestionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Limit => f.write_str("suggestion limit exceeded"),
            Self::NotAtEnd => f.write_str("suggestion requires an end-of-draft cursor"),
            Self::Edit(e) => e.fmt(f),
            Self::Interaction(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for SuggestionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Edit(e) => Some(e),
            Self::Interaction(e) => Some(e),
            _ => None,
        }
    }
}
impl From<EditError> for SuggestionError {
    fn from(value: EditError) -> Self {
        Self::Edit(value)
    }
}
impl From<Error> for SuggestionError {
    fn from(value: Error) -> Self {
        Self::Interaction(value)
    }
}

fn from_sources(
    snapshot: &AnalysisSnapshot,
    sources: &[&str],
) -> Result<Option<Suggestion>, SuggestionError> {
    if sources.len() > MAX_SUGGESTION_SOURCE_ITEMS
        || sources
            .iter()
            .try_fold(0usize, |n, s| n.checked_add(s.len()))
            .is_none_or(|n| n > MAX_SUGGESTION_SOURCE_BYTES)
    {
        return Err(SuggestionError::Limit);
    }
    if snapshot.cursor() != snapshot.text().len() {
        return Err(SuggestionError::NotAtEnd);
    }
    for source in sources {
        if source.starts_with(snapshot.text()) && source.len() > snapshot.text().len() {
            return Ok(Some(Suggestion::new(
                snapshot.revision(),
                &source[snapshot.text().len()..],
            )?));
        }
    }
    Ok(None)
}

/// Select the first caller-ordered value extending the current whole-draft prefix.
pub fn suggest_from_static(
    snapshot: &AnalysisSnapshot,
    values: &[&str],
) -> Result<Option<Suggestion>, SuggestionError> {
    from_sources(snapshot, values)
}
/// Select the newest first history value extending the current whole-draft prefix.
/// Durable history storage, ordering, privacy, and retention remain host-owned.
pub fn suggest_from_history(
    snapshot: &AnalysisSnapshot,
    newest_first: &[&str],
) -> Result<Option<Suggestion>, SuggestionError> {
    from_sources(snapshot, newest_first)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Editor;
    #[test]
    fn helper_returns_only_noncanonical_suffix() {
        let mut e = Editor::new(100, 0);
        e.insert("dep").unwrap();
        let s = suggest_from_static(&e.analysis_snapshot(), &["deploy", "debug"])
            .unwrap()
            .unwrap();
        assert_eq!(s.text(), "loy");
        assert_eq!(e.text(), "dep");
    }
}
