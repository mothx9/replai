//! Optional bounded producers for the low-level completion protocol.
use crate::{AnalysisSnapshot, CompletionCandidate, CompletionError, CompletionSet, DraftRevision};
use std::{
    fmt, io,
    ops::Range,
    path::{Path, PathBuf},
};
use unicode_segmentation::UnicodeSegmentation;

/// Maximum query bytes accepted by generic completion helpers.
pub const MAX_COMPLETION_QUERY_BYTES: usize = 4_096;

/// Case policy for generic prefix and subsequence matching.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MatchCase {
    /// Match Unicode scalar values exactly.
    #[default]
    Sensitive,
    /// Fold ASCII letters only; all other Unicode scalar values remain exact.
    AsciiInsensitive,
}

/// One reusable list item. Its insertion and presentation remain separate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletionItem {
    value: Box<str>,
    label: Box<str>,
    annotation: Option<Box<str>>,
}
impl CompletionItem {
    /// Use the same safe text for insertion and display.
    pub fn new(value: &str) -> Result<Self, CompletionHelperError> {
        Self::labeled(value, value)
    }
    /// Supply distinct insertion and display text.
    pub fn labeled(value: &str, label: &str) -> Result<Self, CompletionHelperError> {
        CompletionCandidate::new(0..0, value, label)?;
        Ok(Self {
            value: value.into(),
            label: label.into(),
            annotation: None,
        })
    }
    /// Add a safe single-line annotation.
    pub fn with_annotation(mut self, annotation: &str) -> Result<Self, CompletionHelperError> {
        CompletionCandidate::new(0..0, &self.value, &self.label)?.with_annotation(annotation)?;
        self.annotation = Some(annotation.into());
        Ok(self)
    }
    /// Exact insertion text.
    pub fn value(&self) -> &str {
        &self.value
    }
    /// Display label.
    pub fn label(&self) -> &str {
        &self.label
    }
    /// Optional display annotation.
    pub fn annotation(&self) -> Option<&str> {
        self.annotation.as_deref()
    }
}

/// A generic helper rejected its query/source, completion payload, or filesystem access.
#[derive(Debug)]
pub enum CompletionHelperError {
    /// Query, source count, or configured output count exceeded a bound.
    Limit,
    /// The host range is outside the snapshot or splits an extended grapheme.
    InvalidRange,
    /// Filesystem traversal failed before a complete result could be produced.
    Io(io::Error),
    /// A generated low-level candidate or set was invalid.
    Completion(CompletionError),
}
impl fmt::Display for CompletionHelperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Limit => f.write_str("completion helper limit exceeded"),
            Self::InvalidRange => f.write_str("completion range is not a snapshot grapheme range"),
            Self::Io(e) => e.fmt(f),
            Self::Completion(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for CompletionHelperError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Completion(e) => Some(e),
            _ => None,
        }
    }
}
impl From<CompletionError> for CompletionHelperError {
    fn from(value: CompletionError) -> Self {
        Self::Completion(value)
    }
}
impl From<io::Error> for CompletionHelperError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

fn validate(
    snapshot: &AnalysisSnapshot,
    range: &Range<usize>,
    query: &str,
    count: usize,
) -> Result<(), CompletionHelperError> {
    if query.len() > MAX_COMPLETION_QUERY_BYTES || count > crate::MAX_COMPLETION_CANDIDATES {
        return Err(CompletionHelperError::Limit);
    }
    let text = snapshot.text();
    if range.start > range.end
        || range.end > text.len()
        || !text.is_char_boundary(range.start)
        || !text.is_char_boundary(range.end)
        || !text.grapheme_indices(true).any(|(i, _)| i == range.start) && range.start != text.len()
        || !text.grapheme_indices(true).any(|(i, _)| i == range.end) && range.end != text.len()
    {
        return Err(CompletionHelperError::InvalidRange);
    }
    Ok(())
}
fn matches_prefix(value: &str, query: &str, case: MatchCase) -> bool {
    match case {
        MatchCase::Sensitive => value.starts_with(query),
        MatchCase::AsciiInsensitive => value
            .get(..query.len())
            .is_some_and(|v| v.eq_ignore_ascii_case(query)),
    }
}
fn candidate(
    range: &Range<usize>,
    item: &CompletionItem,
) -> Result<CompletionCandidate, CompletionError> {
    let mut candidate = CompletionCandidate::new(range.clone(), &item.value, &item.label)?;
    if let Some(annotation) = &item.annotation {
        candidate = candidate.with_annotation(annotation)?;
    }
    Ok(candidate)
}

/// Produce prefix matches in caller order, retaining duplicates.
pub fn complete_prefix(
    snapshot: &AnalysisSnapshot,
    range: Range<usize>,
    query: &str,
    items: &[CompletionItem],
    case: MatchCase,
) -> Result<CompletionSet, CompletionHelperError> {
    validate(snapshot, &range, query, items.len())?;
    let candidates = items
        .iter()
        .filter(|item| matches_prefix(item.value(), query, case))
        .map(|item| candidate(&range, item))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CompletionSet::new(snapshot.revision(), candidates)?)
}

/// Longest shared prefix ending at an extended-grapheme boundary of every value.
pub fn common_grapheme_prefix(values: &[&str]) -> String {
    let Some(first) = values.first() else {
        return String::new();
    };
    let mut end = first.len();
    for value in &values[1..] {
        let common = first
            .bytes()
            .zip(value.bytes())
            .take_while(|(a, b)| a == b)
            .count();
        end = end.min(
            first
                .grapheme_indices(true)
                .map(|(i, _)| i)
                .chain([first.len()])
                .take_while(|i| *i <= common)
                .last()
                .unwrap_or(0),
        );
    }
    first[..end].to_owned()
}

fn fold(c: char, case: MatchCase) -> char {
    if case == MatchCase::AsciiInsensitive {
        c.to_ascii_lowercase()
    } else {
        c
    }
}
fn fuzzy_score(value: &str, query: &str, case: MatchCase) -> Option<(usize, usize)> {
    if query.is_empty() {
        return Some((0, 0));
    }
    let mut source = value.char_indices();
    let mut previous = None;
    let mut start = 0;
    let mut gaps = 0;
    for wanted in query.chars().map(|c| fold(c, case)) {
        let (index, _) = source.find(|(_, got)| fold(*got, case) == wanted)?;
        if let Some(last) = previous {
            gaps += index.saturating_sub(last + 1);
        } else {
            start = index;
        }
        previous = Some(index);
    }
    Some((start, gaps))
}

/// Produce deterministic Unicode-scalar subsequence matches.
/// Lower start offset wins, then fewer byte gaps; ties preserve caller order.
pub fn complete_fuzzy(
    snapshot: &AnalysisSnapshot,
    range: Range<usize>,
    query: &str,
    items: &[CompletionItem],
    case: MatchCase,
) -> Result<CompletionSet, CompletionHelperError> {
    validate(snapshot, &range, query, items.len())?;
    let mut matches = items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            fuzzy_score(item.value(), query, case).map(|score| (score, index, item))
        })
        .collect::<Vec<_>>();
    matches.sort_by_key(|(score, index, _)| (*score, *index));
    let candidates = matches
        .into_iter()
        .map(|(_, _, item)| candidate(&range, item))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CompletionSet::new(snapshot.revision(), candidates)?)
}

/// Bounds and policy for one-level path completion.
#[derive(Clone, Debug)]
pub struct PathCompletionOptions {
    base: PathBuf,
    include_hidden: bool,
    max_candidates: usize,
}
impl PathCompletionOptions {
    /// Resolve relative input below `base`; absolute input ignores it.
    pub fn new(base: impl Into<PathBuf>) -> Self {
        Self {
            base: base.into(),
            include_hidden: false,
            max_candidates: crate::MAX_COMPLETION_CANDIDATES,
        }
    }
    /// Include names whose first scalar is `.`.
    pub fn include_hidden(mut self, include: bool) -> Self {
        self.include_hidden = include;
        self
    }
    /// Set a non-zero result bound no larger than the protocol candidate limit.
    pub fn max_candidates(mut self, maximum: usize) -> Result<Self, CompletionHelperError> {
        if maximum == 0 || maximum > crate::MAX_COMPLETION_CANDIDATES {
            return Err(CompletionHelperError::Limit);
        }
        self.max_candidates = maximum;
        Ok(self)
    }
}

/// Complete one filesystem path component without shell parsing, quoting, or expansion.
/// Non-UTF-8 entry names are skipped. Enumeration and metadata failures are returned.
pub fn complete_path(
    snapshot: &AnalysisSnapshot,
    range: Range<usize>,
    input: &str,
    options: &PathCompletionOptions,
) -> Result<CompletionSet, CompletionHelperError> {
    validate(snapshot, &range, input, options.max_candidates)?;
    let path = Path::new(input);
    let trailing = input.ends_with(std::path::MAIN_SEPARATOR);
    let (typed_parent, prefix) = if trailing {
        (path, "")
    } else {
        (
            path.parent().unwrap_or_else(|| Path::new("")),
            path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
        )
    };
    let directory = if path.is_absolute() {
        typed_parent.to_path_buf()
    } else {
        options.base.join(typed_parent)
    };
    let mut found = Vec::new();
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if (!options.include_hidden && name.starts_with('.')) || !name.starts_with(prefix) {
            continue;
        }
        let metadata = entry.metadata()?;
        let is_dir = metadata.is_dir();
        let mut insertion = typed_parent.join(&name).to_string_lossy().into_owned();
        if is_dir {
            insertion.push(std::path::MAIN_SEPARATOR);
        }
        let annotation = if is_dir {
            "directory"
        } else if entry.file_type()?.is_symlink() {
            "symlink"
        } else {
            "file"
        };
        found.push((insertion, name, annotation));
        if found.len() > options.max_candidates {
            return Err(CompletionHelperError::Limit);
        }
    }
    found.sort_by(|a, b| a.0.cmp(&b.0));
    let items = found
        .into_iter()
        .map(|(value, label, annotation)| {
            CompletionItem::labeled(&value, &label)?.with_annotation(annotation)
        })
        .collect::<Result<Vec<_>, CompletionHelperError>>()?;
    let candidates = items
        .iter()
        .map(|item| candidate(&range, item))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CompletionSet::new(snapshot.revision(), candidates)?)
}

#[allow(dead_code)]
fn _revision(_: DraftRevision) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Editor;
    #[test]
    fn prefix_fuzzy_and_common_prefix_are_deterministic() {
        let mut editor = Editor::new(100, 0);
        editor.insert("bu").unwrap();
        let s = editor.analysis_snapshot();
        let items = [
            CompletionItem::new("build").unwrap(),
            CompletionItem::new("bundle").unwrap(),
            CompletionItem::new("debug").unwrap(),
        ];
        assert_eq!(
            complete_prefix(&s, 0..2, "bu", &items, MatchCase::Sensitive)
                .unwrap()
                .candidates()
                .len(),
            2
        );
        assert_eq!(
            complete_fuzzy(&s, 0..2, "bg", &items, MatchCase::Sensitive)
                .unwrap()
                .candidates()[0]
                .replacement(),
            "debug"
        );
        assert_eq!(
            common_grapheme_prefix(&["cafe\u{301}x", "cafe\u{301}y"]),
            "cafe\u{301}"
        );
    }
}
