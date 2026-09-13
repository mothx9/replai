use crate::{AnalysisOutcome, AnalysisSnapshot, DraftRevision};
use std::{collections::VecDeque, fmt, ops::Range};
use unicode_segmentation::{GraphemeCursor, UnicodeSegmentation};

/// Default number of logical edits retained for undo and redo.
pub const DEFAULT_UNDO_ENTRIES: usize = 256;
/// Maximum payload combined into one contiguous typing/deletion undo group.
pub const MAX_UNDO_GROUP_BYTES: usize = 4_096;

/// Low-level bounds for reversible editing and the private kill register.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EditorLimits {
    undo_entries: usize,
    undo_bytes: usize,
    kill_bytes: usize,
}
impl EditorLimits {
    /// Construct bounds. `undo_bytes` must hold one worst-case replacement.
    pub fn new(
        max_bytes: usize,
        undo_entries: usize,
        undo_bytes: usize,
        kill_bytes: usize,
    ) -> Result<Self, EditError> {
        if undo_entries == 0 || undo_bytes < max_bytes.saturating_mul(2) || kill_bytes > max_bytes {
            return Err(EditError::InvalidConfiguration);
        }
        Ok(Self {
            undo_entries,
            undo_bytes,
            kill_bytes,
        })
    }
    /// Maximum logical records retained independently by undo and redo.
    pub fn undo_entries(self) -> usize {
        self.undo_entries
    }
    /// Maximum replacement payload retained independently by undo and redo.
    pub fn undo_bytes(self) -> usize {
        self.undo_bytes
    }
    /// Maximum UTF-8 bytes retained in the editor-owned kill register.
    pub fn kill_bytes(self) -> usize {
        self.kill_bytes
    }
    fn for_capacity(max_bytes: usize) -> Self {
        Self {
            undo_entries: DEFAULT_UNDO_ENTRIES,
            undo_bytes: max_bytes.saturating_mul(2),
            kill_bytes: max_bytes.min(64 * 1024),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EditGroup {
    Insert,
    Backspace,
    Delete,
    Atomic,
}

#[derive(Debug)]
struct EditRecord {
    start: usize,
    removed: String,
    inserted: String,
    cursor_before: usize,
    cursor_after: usize,
    group: EditGroup,
}
impl EditRecord {
    fn bytes(&self) -> usize {
        self.removed.len() + self.inserted.len()
    }
}

#[derive(Debug)]
struct UndoState {
    undo: VecDeque<EditRecord>,
    redo: VecDeque<EditRecord>,
    undo_bytes: usize,
    redo_bytes: usize,
    limits: EditorLimits,
    group_open: bool,
}
impl UndoState {
    fn new(limits: EditorLimits) -> Self {
        Self {
            undo: VecDeque::new(),
            redo: VecDeque::new(),
            undo_bytes: 0,
            redo_bytes: 0,
            limits,
            group_open: false,
        }
    }
    fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.undo_bytes = 0;
        self.redo_bytes = 0;
        self.group_open = false;
    }
    fn break_group(&mut self) {
        self.group_open = false;
    }
    fn clear_redo(&mut self) {
        self.redo.clear();
        self.redo_bytes = 0;
    }
    fn trim(stack: &mut VecDeque<EditRecord>, bytes: &mut usize, limits: EditorLimits) {
        while stack.len() > limits.undo_entries || *bytes > limits.undo_bytes {
            if let Some(old) = stack.pop_front() {
                *bytes -= old.bytes();
            } else {
                break;
            }
        }
    }
    fn record(&mut self, mut record: EditRecord) {
        self.clear_redo();
        if self.group_open
            && let Some(previous) = self.undo.back_mut()
        {
            let old = previous.bytes();
            let merged = match (previous.group, record.group) {
                (EditGroup::Insert, EditGroup::Insert)
                    if previous.removed.is_empty()
                        && record.removed.is_empty()
                        && previous.start + previous.inserted.len() == record.start
                        && previous.cursor_after == record.cursor_before =>
                {
                    if previous
                        .inserted
                        .len()
                        .saturating_add(record.inserted.len())
                        > MAX_UNDO_GROUP_BYTES
                    {
                        false
                    } else {
                        previous.inserted.push_str(&record.inserted);
                        previous.cursor_after = record.cursor_after;
                        true
                    }
                }
                (EditGroup::Backspace, EditGroup::Backspace)
                    if previous.inserted.is_empty()
                        && record.inserted.is_empty()
                        && record.start + record.removed.len() == previous.start
                        && previous.cursor_after == record.cursor_before =>
                {
                    if previous.removed.len().saturating_add(record.removed.len())
                        > MAX_UNDO_GROUP_BYTES
                    {
                        false
                    } else {
                        record.removed.push_str(&previous.removed);
                        previous.start = record.start;
                        previous.removed = std::mem::take(&mut record.removed);
                        previous.cursor_after = record.cursor_after;
                        true
                    }
                }
                (EditGroup::Delete, EditGroup::Delete)
                    if previous.inserted.is_empty()
                        && record.inserted.is_empty()
                        && previous.start == record.start
                        && previous.cursor_after == record.cursor_before =>
                {
                    if previous.removed.len().saturating_add(record.removed.len())
                        > MAX_UNDO_GROUP_BYTES
                    {
                        false
                    } else {
                        previous.removed.push_str(&record.removed);
                        previous.cursor_after = record.cursor_after;
                        true
                    }
                }
                _ => false,
            };
            if merged {
                self.undo_bytes = self.undo_bytes - old + previous.bytes();
                Self::trim(&mut self.undo, &mut self.undo_bytes, self.limits);
                return;
            }
        }
        self.undo_bytes += record.bytes();
        self.group_open = record.group != EditGroup::Atomic;
        self.undo.push_back(record);
        Self::trim(&mut self.undo, &mut self.undo_bytes, self.limits);
    }
}

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
    /// Editing bounds are internally inconsistent.
    InvalidConfiguration,
    /// A kill would exceed the configured register capacity.
    KillCapacity,
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
            Self::InvalidConfiguration => "invalid editor limits",
            Self::KillCapacity => "kill register capacity exceeded",
        })
    }
}
impl std::error::Error for EditError {}

pub(crate) fn valid_text(text: &str) -> bool {
    text.chars()
        .all(|c| !c.is_control() || c == '\n' || c == '\t')
}

fn word_segment(text: &str) -> bool {
    text.unicode_words().next().is_some()
}
fn grapheme_floor(text: &str, index: usize) -> usize {
    if index == 0 || index == text.len() {
        return index;
    }
    let mut cursor = GraphemeCursor::new(index, text.len(), true);
    if cursor.is_boundary(text, 0).expect("complete text") {
        index
    } else {
        cursor
            .prev_boundary(text, 0)
            .expect("complete text")
            .unwrap_or(0)
    }
}
fn grapheme_ceil(text: &str, index: usize) -> usize {
    if index == 0 || index == text.len() {
        return index;
    }
    let mut cursor = GraphemeCursor::new(index, text.len(), true);
    if cursor.is_boundary(text, 0).expect("complete text") {
        index
    } else {
        cursor
            .next_boundary(text, 0)
            .expect("complete text")
            .unwrap_or(text.len())
    }
}
fn word_left_boundary(text: &str, cursor: usize) -> usize {
    let semantic = text[..cursor]
        .split_word_bound_indices()
        .rev()
        .find_map(|(offset, segment)| word_segment(segment).then_some(offset))
        .unwrap_or(0);
    grapheme_floor(text, semantic)
}
fn word_right_boundary(text: &str, cursor: usize) -> usize {
    let semantic = text[cursor..]
        .split_word_bound_indices()
        .find_map(|(offset, segment)| {
            word_segment(segment).then_some(cursor + offset + segment.len())
        })
        .unwrap_or(text.len());
    grapheme_ceil(text, semantic)
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
    undo: Box<UndoState>,
    kill: String,
}

impl Editor {
    /// Create an empty editor with byte and history-entry limits. Zero is allowed.
    pub fn new(max_bytes: usize, history_entries: usize) -> Self {
        Self::with_limits(
            max_bytes,
            history_entries,
            EditorLimits::for_capacity(max_bytes),
        )
        .expect("default editor limits are coherent")
    }
    /// Create an editor with explicit reversible-editing and kill-register bounds.
    pub fn with_limits(
        max_bytes: usize,
        history_entries: usize,
        limits: EditorLimits,
    ) -> Result<Self, EditError> {
        EditorLimits::new(
            max_bytes,
            limits.undo_entries,
            limits.undo_bytes,
            limits.kill_bytes,
        )?;
        Ok(Self {
            revision: DraftRevision::INITIAL,
            text: String::new(),
            cursor: 0,
            limit: max_bytes,
            history: VecDeque::new(),
            history_limit: history_entries,
            selected: None,
            draft: None,
            undo: Box::new(UndoState::new(limits)),
            kill: String::new(),
        })
    }
    /// Active reversible-editing and kill-register bounds.
    pub fn limits(&self) -> EditorLimits {
        self.undo.limits
    }
    /// Number of retained undo records and their replacement payload bytes.
    pub fn undo_usage(&self) -> (usize, usize) {
        (self.undo.undo.len(), self.undo.undo_bytes)
    }
    /// Number of retained redo records and their replacement payload bytes.
    pub fn redo_usage(&self) -> (usize, usize) {
        (self.undo.redo.len(), self.undo.redo_bytes)
    }
    /// Current kill-register text. It is editor state, not a system clipboard.
    pub fn kill_register(&self) -> &str {
        &self.kill
    }
    /// Erase the private kill register without changing draft identity.
    pub fn clear_kill_register(&mut self) {
        self.kill.clear();
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
        self.undo.clear();
        self.selected = None;
        self.draft = None;
    }
    fn move_cursor(&mut self, cursor: usize) {
        if cursor != self.cursor {
            self.revision.advance();
            self.cursor = cursor;
            self.undo.break_group();
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
    pub(crate) fn validate_replacement(
        &self,
        range: &Range<usize>,
        text: &str,
    ) -> Result<(), EditError> {
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
        Ok(())
    }
    /// Replace a grapheme-aligned byte range atomically. Cursor follows replacement.
    pub fn replace(&mut self, range: Range<usize>, text: &str) -> Result<(), EditError> {
        self.replace_group(range, text, EditGroup::Atomic)
    }
    fn replace_group(
        &mut self,
        range: Range<usize>,
        text: &str,
        group: EditGroup,
    ) -> Result<(), EditError> {
        self.validate_replacement(&range, text)?;
        let wanted = range.start + text.len();
        if &self.text[range.clone()] == text && wanted == self.cursor {
            return Ok(());
        }
        let record = EditRecord {
            start: range.start,
            removed: self.text[range.clone()].to_owned(),
            inserted: text.to_owned(),
            cursor_before: self.cursor,
            cursor_after: wanted,
            group,
        };
        self.revision.advance();
        self.text.replace_range(range, text);
        // Joining marks/ZWJ may merge both sides of the insertion.
        self.cursor = wanted;
        self.snap_cursor();
        let mut record = record;
        record.cursor_after = self.cursor;
        self.undo.record(record);
        Ok(())
    }
    /// Insert text at the cursor without interpreting commands or trimming it.
    pub fn insert(&mut self, text: &str) -> Result<(), EditError> {
        self.replace_group(self.cursor..self.cursor, text, EditGroup::Insert)
    }
    pub(crate) fn insert_transaction(&mut self, text: &str) -> Result<(), EditError> {
        self.replace_group(self.cursor..self.cursor, text, EditGroup::Atomic)
    }
    pub(crate) fn break_undo_group(&mut self) {
        self.undo.break_group();
    }
    pub(crate) fn replace_search_result(&mut self, text: &str) -> Result<(), EditError> {
        self.validate_replacement(&(0..self.text.len()), text)?;
        if self.text == text && self.cursor == text.len() {
            self.revision.advance();
            self.undo.break_group();
        } else {
            self.replace_group(0..self.text.len(), text, EditGroup::Atomic)?;
        }
        self.selected = None;
        self.draft = None;
        Ok(())
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
        self.undo.clear();
    }
    /// Move to the previous logical LF-delimited line at the current display column.
    /// Returns false only at the first logical line. Short lines clamp to their end;
    /// tabs use four-cell stops relative to the logical line, independently of prompt.
    /// Soft wraps remain navigable with Left/Right. No history policy is applied here.
    pub fn line_up(&mut self) -> bool {
        self.undo.break_group();
        self.vertical(false)
    }
    /// Move to the next logical line using the same geometry as [`Self::line_up`].
    /// Returns false at the last line; cursor changes advance the draft revision.
    pub fn line_down(&mut self) -> bool {
        self.undo.break_group();
        self.vertical(true)
    }
    fn vertical(&mut self, down: bool) -> bool {
        let start = self.text[..self.cursor].rfind('\n').map_or(0, |i| i + 1);
        let next = self.text[self.cursor..]
            .find('\n')
            .map(|i| self.cursor + i + 1);
        let target = if down {
            let Some(next) = next else { return false };
            next
        } else {
            if start == 0 {
                return false;
            }
            self.text[..start - 1].rfind('\n').map_or(0, |i| i + 1)
        };
        let width = |g: &str, col: usize| {
            if g == "\t" {
                4 - col % 4
            } else {
                crate::width::cells(g)
            }
        };
        let desired = self.text[start..self.cursor]
            .graphemes(true)
            .fold(0, |col, g| col + width(g, col));
        let mut col = 0;
        let mut cursor = target;
        for g in self.text[target..].graphemes(true) {
            if g == "\n" || col + width(g, col) > desired {
                break;
            }
            col += width(g, col);
            cursor += g.len();
        }
        self.move_cursor(cursor);
        true
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
    /// Move to the beginning of the preceding/current UAX #29 word.
    pub fn word_left(&mut self) {
        self.move_cursor(word_left_boundary(&self.text, self.cursor));
    }
    /// Move to the end of the following/current UAX #29 word.
    pub fn word_right(&mut self) {
        self.move_cursor(word_right_boundary(&self.text, self.cursor));
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
        self.replace_group(start..end, "", EditGroup::Backspace)
            .expect("existing grapheme is valid");
    }
    /// Remove the following extended grapheme, or do nothing at the end.
    pub fn delete(&mut self) {
        let start = self.cursor;
        let Some(g) = self.text[start..].graphemes(true).next() else {
            return;
        };
        let end = start + g.len();
        self.replace_group(start..end, "", EditGroup::Delete)
            .expect("existing grapheme is valid");
    }
    /// Delete backward through one Unicode word and intervening separators.
    pub fn delete_word_backward(&mut self) {
        let start = word_left_boundary(&self.text, self.cursor);
        if start != self.cursor {
            self.replace_group(start..self.cursor, "", EditGroup::Atomic)
                .expect("word boundary is valid");
        }
    }
    /// Delete forward through one Unicode word and intervening separators.
    pub fn delete_word_forward(&mut self) {
        let end = word_right_boundary(&self.text, self.cursor);
        if end != self.cursor {
            self.replace_group(self.cursor..end, "", EditGroup::Atomic)
                .expect("word boundary is valid");
        }
    }
    /// Undo one grouped canonical edit. Content and cursor return with a fresh revision.
    pub fn undo(&mut self) -> bool {
        let Some(record) = self.undo.undo.back() else {
            self.undo.break_group();
            return false;
        };
        self.revision.advance();
        let bytes = record.bytes();
        let record = self.undo.undo.pop_back().unwrap();
        self.undo.undo_bytes -= bytes;
        let end = record.start + record.inserted.len();
        self.text.replace_range(record.start..end, &record.removed);
        self.cursor = record.cursor_before;
        self.undo.redo_bytes += bytes;
        self.undo.redo.push_back(record);
        UndoState::trim(
            &mut self.undo.redo,
            &mut self.undo.redo_bytes,
            self.undo.limits,
        );
        self.undo.break_group();
        true
    }
    /// Redo one just-undone canonical edit with a fresh revision.
    pub fn redo(&mut self) -> bool {
        let Some(record) = self.undo.redo.back() else {
            self.undo.break_group();
            return false;
        };
        self.revision.advance();
        let bytes = record.bytes();
        let record = self.undo.redo.pop_back().unwrap();
        self.undo.redo_bytes -= bytes;
        let end = record.start + record.removed.len();
        self.text.replace_range(record.start..end, &record.inserted);
        self.cursor = record.cursor_after;
        self.undo.undo_bytes += bytes;
        self.undo.undo.push_back(record);
        UndoState::trim(
            &mut self.undo.undo,
            &mut self.undo.undo_bytes,
            self.undo.limits,
        );
        self.undo.break_group();
        true
    }
    fn kill_range(&mut self, range: Range<usize>) -> Result<(), EditError> {
        if range.is_empty() {
            self.undo.break_group();
            return Ok(());
        }
        if range.len() > self.undo.limits.kill_bytes {
            return Err(EditError::KillCapacity);
        }
        let killed = self.text[range.clone()].to_owned();
        self.replace_group(range, "", EditGroup::Atomic)?;
        self.kill = killed;
        Ok(())
    }
    /// Kill one Unicode word backward into the bounded private register.
    pub fn kill_word_backward(&mut self) -> Result<(), EditError> {
        self.kill_range(word_left_boundary(&self.text, self.cursor)..self.cursor)
    }
    /// Kill one Unicode word forward into the bounded private register.
    pub fn kill_word_forward(&mut self) -> Result<(), EditError> {
        self.kill_range(self.cursor..word_right_boundary(&self.text, self.cursor))
    }
    /// Kill from the cursor to the end of the current logical line.
    pub fn kill_line_end(&mut self) -> Result<(), EditError> {
        let end = self.text[self.cursor..]
            .find('\n')
            .map_or(self.text.len(), |i| self.cursor + i);
        self.kill_range(self.cursor..end)
    }
    /// Kill from the cursor to the beginning of the current logical line.
    pub fn kill_line_start(&mut self) -> Result<(), EditError> {
        let start = self.text[..self.cursor].rfind('\n').map_or(0, |i| i + 1);
        self.kill_range(start..self.cursor)
    }
    /// Insert the private kill register atomically. An empty register is a no-op.
    pub fn yank(&mut self) -> Result<(), EditError> {
        if self.kill.is_empty() {
            self.undo.break_group();
            return Ok(());
        }
        let text = self.kill.clone();
        self.replace_group(self.cursor..self.cursor, &text, EditGroup::Atomic)
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
        self.undo.break_group();
        if self.history.len() == self.history_limit {
            self.history.pop_front();
        }
        self.history.push_back(text.to_owned());
        Ok(())
    }
    /// Recall the previous entry, preserving the current draft and its cursor.
    pub fn history_up(&mut self) {
        if self.history.is_empty() || self.selected == Some(0) {
            self.undo.break_group();
            return;
        }
        let index = self.selected.map_or(self.history.len() - 1, |i| i - 1);
        if self.text != self.history[index] || self.cursor != self.history[index].len() {
            self.revision.advance();
        }
        self.undo.clear();
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
            self.undo.break_group();
            return;
        };
        if index + 1 < self.history.len() {
            if self.text != self.history[index + 1] || self.cursor != self.history[index + 1].len()
            {
                self.revision.advance();
            }
            self.undo.clear();
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
            self.undo.clear();
            if let Some((text, cursor)) = self.draft.take() {
                self.text = text;
                self.cursor = cursor;
            }
            self.selected = None;
        }
    }
    pub(crate) fn history_newest(&self) -> impl Iterator<Item = &str> {
        self.history.iter().rev().map(String::as_str)
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
