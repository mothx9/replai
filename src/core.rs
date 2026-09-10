use crate::{AnalysisOutcome, AnalysisSnapshot, DraftRevision};
use std::{collections::VecDeque, fmt, ops::Range};
use unicode_segmentation::{GraphemeCursor, UnicodeSegmentation};

/// A rejected edit. Rejection leaves the text and cursor unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EditError {
    /// The configured UTF-8 byte capacity would be exceeded.
    Capacity,
    /// Text contains a control character other than LF or TAB.
    InvalidText,
    /// A replacement range is reversed, outside the buffer, or splits a grapheme.
    InvalidRange,
    /// The host configured zero history entries.
    HistoryDisabled,
    /// Input was not valid, complete UTF-8.
    InvalidUtf8,
    /// An unknown or incomplete terminal sequence was rejected.
    InvalidSequence,
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Capacity => "input capacity exceeded",
            Self::InvalidText => "unsupported control character",
            Self::InvalidRange => "replacement must use ordered grapheme boundaries",
            Self::HistoryDisabled => "history is disabled",
            Self::InvalidUtf8 => "invalid or incomplete UTF-8",
            Self::InvalidSequence => "unknown or incomplete terminal sequence",
        })
    }
}
impl std::error::Error for EditError {}

pub(crate) fn valid_text(text: &str) -> bool {
    text.chars()
        .all(|c| !c.is_control() || c == '\n' || c == '\t')
}

/// Deterministic bounded text editing, with extended-grapheme cursor movement.
///
/// Offsets are UTF-8 byte offsets at extended grapheme boundaries. Home and End
/// refer to the entire input, including multiline input. History admission is
/// explicit; navigating an edited recalled entry never changes stored history.
/// Analysis revision bookkeeping allocates nothing. On exhaustion of the private
/// 128-bit revision domain, a state-changing operation panics before mutation.
/// No-op and rejected edits retain their revision.
#[derive(Debug)]
pub struct Editor {
    revision: DraftRevision,
    text: String,
    cursor: usize,
    limit: usize,
    history: VecDeque<String>,
    history_limit: usize,
    selected: Option<usize>,
    draft: Option<(String, usize)>,
}

impl Editor {
    /// Create an empty editor with byte and history-entry limits. Zero is allowed.
    pub fn new(max_bytes: usize, history_entries: usize) -> Self {
        Self {
            revision: DraftRevision::INITIAL,
            text: String::new(),
            cursor: 0,
            limit: max_bytes,
            history: VecDeque::new(),
            history_limit: history_entries,
            selected: None,
            draft: None,
        }
    }
    /// Current analysis identity, scoped to this retained editor.
    pub fn revision(&self) -> DraftRevision {
        self.revision
    }
    /// Copy one coherent view for retained host analysis; clones share the text.
    pub fn analysis_snapshot(&self) -> AnalysisSnapshot {
        AnalysisSnapshot::new(self.revision, &self.text, self.cursor)
    }
    /// Replace only if the originating revision is still current.
    ///
    /// Staleness is checked before validation; stale input cannot change text,
    /// cursor or history navigation, even if its range or text is invalid.
    pub fn replace_at(
        &mut self,
        revision: DraftRevision,
        range: Range<usize>,
        text: &str,
    ) -> Result<AnalysisOutcome, EditError> {
        if revision != self.revision {
            return Ok(AnalysisOutcome::Stale);
        }
        self.replace(range, text)?;
        Ok(AnalysisOutcome::Applied)
    }
    // Checked before every mutation. Exhaustion panics before altering state;
    // wrapping must never make an ancient snapshot eligible again.
    pub(crate) fn end_draft(&mut self) {
        self.revision.advance();
    }
    fn move_cursor(&mut self, cursor: usize) {
        if cursor != self.cursor {
            self.revision.advance();
            self.cursor = cursor;
        }
    }
    /// Current input, without application interpretation.
    pub fn text(&self) -> &str {
        &self.text
    }
    /// Cursor as a UTF-8 byte offset at an extended grapheme boundary.
    pub fn cursor(&self) -> usize {
        self.cursor
    }
    /// Configured maximum input size in UTF-8 bytes.
    pub fn capacity(&self) -> usize {
        self.limit
    }
    fn boundary(&self, index: usize) -> bool {
        index == 0
            || index == self.text.len()
            || (self.text.is_char_boundary(index)
                && GraphemeCursor::new(index, self.text.len(), true)
                    .is_boundary(&self.text, 0)
                    .expect("complete editor text supplies all grapheme context"))
    }
    /// Replace a grapheme-aligned byte range atomically. Cursor follows replacement.
    pub fn replace(&mut self, range: Range<usize>, text: &str) -> Result<(), EditError> {
        if range.start > range.end
            || range.end > self.text.len()
            || !self.boundary(range.start)
            || !self.boundary(range.end)
        {
            return Err(EditError::InvalidRange);
        }
        if !valid_text(text) {
            return Err(EditError::InvalidText);
        }
        let remaining = self.text.len() - range.len();
        if text.len() > self.limit - remaining {
            return Err(EditError::Capacity);
        }
        let wanted = range.start + text.len();
        if &self.text[range.clone()] == text && wanted == self.cursor {
            return Ok(());
        }
        self.revision.advance();
        self.text.replace_range(range, text);
        // Joining marks/ZWJ may merge both sides of the insertion.
        self.cursor = wanted;
        self.snap_cursor();
        Ok(())
    }
    /// Insert text at the cursor without interpreting commands or trimming it.
    pub fn insert(&mut self, text: &str) -> Result<(), EditError> {
        self.replace(self.cursor..self.cursor, text)
    }
    /// Clear the active input and navigation draft; retain admitted history.
    pub fn clear(&mut self) {
        if !self.text.is_empty() {
            self.revision.advance();
        }
        self.text.clear();
        self.cursor = 0;
        self.selected = None;
        self.draft = None;
    }
    /// Move left one extended grapheme, stopping at the beginning.
    pub fn left(&mut self) {
        let cursor = self.text[..self.cursor]
            .grapheme_indices(true)
            .next_back()
            .map_or(0, |(i, _)| i);
        self.move_cursor(cursor);
    }
    /// Move right one extended grapheme, stopping at the end.
    pub fn right(&mut self) {
        if let Some(g) = self.text[self.cursor..].graphemes(true).next() {
            self.move_cursor(self.cursor + g.len());
        }
    }
    /// Move to the beginning of the entire input.
    pub fn home(&mut self) {
        self.move_cursor(0);
    }
    /// Move to the end of the entire input.
    pub fn end(&mut self) {
        self.move_cursor(self.text.len());
    }
    /// Remove the preceding extended grapheme, or do nothing at the beginning.
    pub fn backspace(&mut self) {
        let end = self.cursor;
        if end == 0 {
            return;
        }
        let start = self.text[..end]
            .grapheme_indices(true)
            .next_back()
            .unwrap()
            .0;
        self.revision.advance();
        self.cursor = start;
        self.text.drain(start..end);
        self.snap_cursor();
    }
    /// Remove the following extended grapheme, or do nothing at the end.
    pub fn delete(&mut self) {
        let start = self.cursor;
        let Some(g) = self.text[start..].graphemes(true).next() else {
            return;
        };
        let end = start + g.len();
        self.revision.advance();
        self.text.drain(start..end);
        self.snap_cursor();
    }
    fn snap_cursor(&mut self) {
        if self.cursor == 0 || self.cursor == self.text.len() {
            return;
        }
        // Ask for the context of this boundary instead of traversing the draft
        // from byte zero. Full text remains available for RI/ZWJ/Indic rules;
        // long contextual graphemes are deliberately not assumed constant-time.
        let mut cursor = GraphemeCursor::new(self.cursor, self.text.len(), true);
        if !cursor
            .is_boundary(&self.text, 0)
            .expect("complete editor text supplies all grapheme context")
        {
            self.cursor = cursor
                .next_boundary(&self.text, 0)
                .expect("complete editor text supplies the following boundary")
                .unwrap_or(self.text.len());
        }
    }
    /// Admit one host-selected history entry; evict the oldest at capacity.
    ///
    /// No deduplication, trimming, persistence or automatic admission occurs.
    pub fn admit_history(&mut self, text: &str) -> Result<(), EditError> {
        if !valid_text(text) {
            return Err(EditError::InvalidText);
        }
        if text.len() > self.limit {
            return Err(EditError::Capacity);
        }
        if self.history_limit == 0 {
            return Err(EditError::HistoryDisabled);
        }
        self.selected = None;
        self.draft = None;
        if self.history.len() == self.history_limit {
            self.history.pop_front();
        }
        self.history.push_back(text.to_owned());
        Ok(())
    }
    /// Recall the previous entry, preserving the current draft and its cursor.
    pub fn history_up(&mut self) {
        if self.history.is_empty() || self.selected == Some(0) {
            return;
        }
        let index = self.selected.map_or(self.history.len() - 1, |i| i - 1);
        if self.text != self.history[index] || self.cursor != self.history[index].len() {
            self.revision.advance();
        }
        if self.selected.is_none() {
            self.draft = Some((self.text.clone(), self.cursor));
        }
        self.selected = Some(index);
        self.text.clone_from(&self.history[index]);
        self.cursor = self.text.len();
    }
    /// Recall the next entry, or return to the original draft and cursor.
    pub fn history_down(&mut self) {
        let Some(index) = self.selected else {
            return;
        };
        if index + 1 < self.history.len() {
            if self.text != self.history[index + 1] || self.cursor != self.history[index + 1].len()
            {
                self.revision.advance();
            }
            self.selected = Some(index + 1);
            self.text.clone_from(&self.history[index + 1]);
            self.cursor = self.text.len();
        } else {
            if self
                .draft
                .as_ref()
                .is_some_and(|(text, cursor)| *text != self.text || *cursor != self.cursor)
            {
                self.revision.advance();
            }
            if let Some((text, cursor)) = self.draft.take() {
                self.text = text;
                self.cursor = cursor;
            }
            self.selected = None;
        }
    }
}

#[cfg(test)]
mod analysis_tests {
    use super::*;
    #[test]
    fn exhausted_editor_refuses_before_visible_or_navigation_mutation() {
        for operation in 0..12 {
            let mut e = Editor::new(64, 2);
            e.insert("draft").unwrap();
            e.admit_history("older").unwrap();
            e.history_up();
            e.left();
            if operation == 11 {
                e.selected = None;
            }
            e.revision = DraftRevision::EXHAUSTED;
            let before = format!("{e:?}");
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match operation {
                    0 => e.insert("x").unwrap(),
                    1 => e.clear(),
                    2 => e.left(),
                    3 => e.right(),
                    4 => e.home(),
                    5 => e.end(),
                    6 => e.backspace(),
                    7 => e.delete(),
                    8 => e.history_down(),
                    9 => e.replace(0..1, "O").unwrap(),
                    10 => e.end_draft(),
                    _ => e.history_up(),
                }));
            assert!(result.is_err());
            assert_eq!(format!("{e:?}"), before);
        }
    }
}
