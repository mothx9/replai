//! Bounded host-fed history views for deterministic in-editor search.
use crate::core::valid_text;
use std::fmt;

/// Default maximum history entries retained in one external search view.
pub const DEFAULT_HISTORY_SEARCH_ENTRIES: usize = 1_024;
/// Default maximum UTF-8 bytes retained in one external search view.
pub const DEFAULT_HISTORY_SEARCH_BYTES: usize = 1 << 20;
/// Default maximum UTF-8 bytes in a reverse-search query.
pub const DEFAULT_HISTORY_QUERY_BYTES: usize = 4_096;

/// Explicit bounds for one host-provided history-search view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HistorySearchLimits {
    entries: usize,
    bytes: usize,
    query_bytes: usize,
}
impl HistorySearchLimits {
    /// Construct non-zero entry, retained-byte and query-byte bounds.
    pub fn new(entries: usize, bytes: usize, query_bytes: usize) -> Result<Self, HistoryError> {
        if entries == 0 || bytes == 0 || query_bytes == 0 {
            return Err(HistoryError::InvalidLimits);
        }
        Ok(Self {
            entries,
            bytes,
            query_bytes,
        })
    }
    /// Maximum entries inspected by one search action.
    pub fn entries(self) -> usize {
        self.entries
    }
    /// Maximum total UTF-8 bytes retained from the provider.
    pub fn bytes(self) -> usize {
        self.bytes
    }
    /// Maximum UTF-8 bytes in a search query.
    pub fn query_bytes(self) -> usize {
        self.query_bytes
    }
}
impl Default for HistorySearchLimits {
    fn default() -> Self {
        Self {
            entries: DEFAULT_HISTORY_SEARCH_ENTRIES,
            bytes: DEFAULT_HISTORY_SEARCH_BYTES,
            query_bytes: DEFAULT_HISTORY_QUERY_BYTES,
        }
    }
}

/// Host-owned source queried synchronously while constructing a bounded view.
///
/// Index zero is the newest entry. The host maps its own storage errors to
/// [`HistoryProviderError`]. REPLAI never retains this provider or calls it from
/// the terminal input path; database/network scheduling remains host-owned.
pub trait HistoryProvider {
    /// Return one newest-first entry, or `None` after the oldest available entry.
    fn entry(&mut self, newest_index: usize) -> Result<Option<String>, HistoryProviderError>;
}

/// Opaque provider failure. Host error text is deliberately not presented.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HistoryProviderError;
impl fmt::Display for HistoryProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("history provider failed")
    }
}
impl std::error::Error for HistoryProviderError {}

/// Failure to construct or install a bounded history-search view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HistoryError {
    /// A configured bound was zero.
    InvalidLimits,
    /// A provider entry contains a disallowed control character.
    InvalidText,
    /// One entry exceeds the retained-byte bound.
    EntryTooLarge,
    /// The provider reported a failure.
    Provider,
    /// The view cannot be installed while search is active.
    SearchActive,
}
impl fmt::Display for HistoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidLimits => "history search limits must be non-zero",
            Self::InvalidText => "history entry contains unsupported control text",
            Self::EntryTooLarge => "history entry exceeds the retained search bound",
            Self::Provider => "history provider failed",
            Self::SearchActive => "history search is active",
        })
    }
}
impl std::error::Error for HistoryError {}

/// Immutable bounded copy used by REPLAI search mechanics.
///
/// Provider storage remains host-owned. Construction is atomic: failure returns
/// no view, and hitting the aggregate entry/byte bound truncates at the oldest
/// edge rather than retaining partially validated data.
#[derive(Clone, Debug)]
pub struct HistorySearchSource {
    entries: Box<[String]>,
    bytes: usize,
    limits: HistorySearchLimits,
}
impl HistorySearchSource {
    /// Materialize a newest-first bounded view from host-owned storage.
    pub fn from_provider(
        provider: &mut impl HistoryProvider,
        limits: HistorySearchLimits,
    ) -> Result<Self, HistoryError> {
        let mut entries = Vec::with_capacity(limits.entries.min(64));
        let mut bytes = 0usize;
        for index in 0..limits.entries {
            let Some(entry) = provider.entry(index).map_err(|_| HistoryError::Provider)? else {
                break;
            };
            if !valid_text(&entry) {
                return Err(HistoryError::InvalidText);
            }
            if entry.len() > limits.bytes {
                return Err(HistoryError::EntryTooLarge);
            }
            let Some(total) = bytes.checked_add(entry.len()) else {
                break;
            };
            if total > limits.bytes {
                break;
            }
            bytes = total;
            entries.push(entry);
        }
        Ok(Self {
            entries: entries.into_boxed_slice(),
            bytes,
            limits,
        })
    }
    /// Build from a newest-first iterator using the same provider contract.
    pub fn from_entries<I, S>(entries: I, limits: HistorySearchLimits) -> Result<Self, HistoryError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        struct IterProvider<I>(I);
        impl<I, S> HistoryProvider for IterProvider<I>
        where
            I: Iterator<Item = S>,
            S: Into<String>,
        {
            fn entry(&mut self, _index: usize) -> Result<Option<String>, HistoryProviderError> {
                Ok(self.0.next().map(Into::into))
            }
        }
        Self::from_provider(&mut IterProvider(entries.into_iter()), limits)
    }
    /// Number of retained entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    /// Whether no entries were retained.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    /// Retained provider payload bytes.
    pub fn retained_bytes(&self) -> usize {
        self.bytes
    }
    /// Limits governing this view and its search query.
    pub fn limits(&self) -> HistorySearchLimits {
        self.limits
    }
    pub(crate) fn entries(&self) -> &[String] {
        &self.entries
    }
}

#[derive(Debug)]
pub(crate) struct HistorySearchState {
    pub revision: crate::DraftRevision,
    pub query: String,
    pub entries: Box<[String]>,
    pub selected: Option<usize>,
    pub query_limit: usize,
}
impl HistorySearchState {
    pub fn new(revision: crate::DraftRevision, entries: Vec<String>, query_limit: usize) -> Self {
        let mut state = Self {
            revision,
            query: String::new(),
            entries: entries.into_boxed_slice(),
            selected: None,
            query_limit,
        };
        state.select_from(0);
        state
    }
    fn select_from(&mut self, start: usize) {
        self.selected =
            (start..self.entries.len()).find(|&i| self.entries[i].contains(&self.query));
    }
    pub fn older(&mut self) {
        self.select_from(self.selected.map_or(0, |i| i + 1));
    }
    pub fn newer(&mut self) {
        let end = self.selected.unwrap_or(self.entries.len());
        self.selected = (0..end)
            .rev()
            .find(|&i| self.entries[i].contains(&self.query));
    }
    pub fn insert_query(&mut self, text: &str) -> Result<(), crate::EditError> {
        if !valid_text(text) || text.contains(['\n', '\t']) {
            return Err(crate::EditError::InvalidText);
        }
        if text.len() > self.query_limit.saturating_sub(self.query.len()) {
            return Err(crate::EditError::Capacity);
        }
        self.query.push_str(text);
        self.select_from(0);
        Ok(())
    }
    pub fn backspace(&mut self) {
        if let Some((index, _)) = self.query.grapheme_indices(true).next_back() {
            self.query.truncate(index);
            self.select_from(0);
        }
    }
    pub fn clear_query(&mut self) {
        self.query.clear();
        self.select_from(0);
    }
    pub fn selected_text(&self) -> Option<&str> {
        self.selected.map(|i| self.entries[i].as_str())
    }
    pub fn delete_query_word(&mut self) {
        let end = self.query.len();
        let start = self.query[..end]
            .split_word_bound_indices()
            .rev()
            .find_map(|(offset, segment)| {
                segment.unicode_words().next().is_some().then_some(offset)
            })
            .unwrap_or(0);
        self.query.truncate(start);
        self.select_from(0);
    }
    pub fn frame(
        &self,
        editor: &crate::Editor,
        prompt: &crate::Prompt,
        size: (usize, usize),
        analysis: Option<&crate::AnalysisPresentation>,
    ) -> crate::presentation::Frame {
        use crate::{
            Role,
            presentation::{Frame, Line, Point, Run},
        };
        let mut frame = Frame::analyzed(
            editor,
            prompt,
            size.0,
            size.1.saturating_sub(1).max(2),
            analysis,
        );
        let match_text = self
            .selected_text()
            .unwrap_or("no match")
            .replace('\n', "↵")
            .replace('\t', "⇥");
        let text = crate::completion::clip(
            &format!("? '{}' > {}", self.query, match_text),
            size.0.saturating_sub(1),
        );
        let width = crate::width::cells(&text);
        frame.lines.push(Line(vec![
            Run::Style(Role::Dim),
            Run::Text(text),
            Run::Style(Role::Default),
        ]));
        frame.widths.push(width);
        frame.rows = size.1;
        frame.end = Point {
            row: frame.lines.len() - 1,
            col: width,
        };
        frame
    }
}

use unicode_segmentation::UnicodeSegmentation;
