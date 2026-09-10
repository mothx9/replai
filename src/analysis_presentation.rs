//! Revision-bound derived display, separate from canonical editor storage.
use crate::{DraftRevision, EditError, Error, Role};
use std::{fmt, ops::Range};
use unicode_segmentation::UnicodeSegmentation;

/// Maximum ordered spans in one presentation result.
pub const MAX_ANALYSIS_SPANS: usize = 4096;
/// Maximum safe single-line hint payload in UTF-8 bytes.
pub const MAX_HINT_BYTES: usize = 4096;

/// A nonempty UTF-8 byte range styled with a generic role, never a token class.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalysisSpan {
    range: Range<usize>,
    role: Role,
}
impl AnalysisSpan {
    /// Construct a nonempty range. Delivery validates snapshot grapheme boundaries.
    pub fn new(range: Range<usize>, role: Role) -> Result<Self, AnalysisPresentationError> {
        if range.start >= range.end {
            return Err(AnalysisPresentationError::InvalidSpans);
        }
        Ok(Self { range, role })
    }
    /// Byte range in the originating snapshot.
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
    /// Generic host-selected appearance.
    pub fn role(&self) -> Role {
        self.role
    }
}
/// Safe non-canonical suffix. It has no acceptance or submission semantics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hint {
    text: Box<str>,
    role: Role,
}
impl Hint {
    /// Nonempty, at most 4096 bytes; no controls, line breaks or hidden controls.
    pub fn new(text: &str, role: Role) -> Result<Self, AnalysisPresentationError> {
        if text.len() > MAX_HINT_BYTES {
            return Err(AnalysisPresentationError::Limit);
        }
        if text.is_empty()
            || text
                .chars()
                .any(|c| c.is_control() || crate::completion::hidden_control(c))
        {
            return Err(AnalysisPresentationError::InvalidHint);
        }
        Ok(Self {
            text: text.into(),
            role,
        })
    }
    /// Complete hint, including any portion clipped from the visible suffix.
    pub fn text(&self) -> &str {
        &self.text
    }
    /// Generic host-selected appearance; textual delimiters also distinguish the hint.
    pub fn role(&self) -> Role {
        self.role
    }
}
/// One whole immutable presentation for one retained editor's revision domain.
/// Cloning copies its bounded payload; it does not retain or copy the draft.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalysisPresentation {
    revision: DraftRevision,
    spans: Box<[AnalysisSpan]>,
    hint: Option<Hint>,
}
impl AnalysisPresentation {
    /// Validate count and ordering before retaining payload. Adjacent ranges are
    /// allowed; overlap, empty and reversed ranges are rejected. No sorting/merging.
    /// Empty spans with no hint explicitly clear current presentation on delivery.
    pub fn new(
        revision: DraftRevision,
        spans: Vec<AnalysisSpan>,
        hint: Option<Hint>,
    ) -> Result<Self, AnalysisPresentationError> {
        if spans.len() > MAX_ANALYSIS_SPANS {
            return Err(AnalysisPresentationError::Limit);
        }
        if spans.windows(2).any(|w| w[0].range.end > w[1].range.start) {
            return Err(AnalysisPresentationError::InvalidSpans);
        }
        Ok(Self {
            revision,
            spans: spans.into_boxed_slice(),
            hint,
        })
    }
    /// Exact draft provenance, not host context or analysis-job priority.
    pub fn revision(&self) -> DraftRevision {
        self.revision
    }
    /// Ordered nonoverlapping generic styles over canonical text.
    pub fn spans(&self) -> &[AnalysisSpan] {
        &self.spans
    }
    /// Optional non-canonical suffix.
    pub fn hint(&self) -> Option<&Hint> {
        self.hint.as_ref()
    }
    pub(crate) fn validate(&self, text: &str) -> Result<(), AnalysisPresentationError> {
        // One monotonic boundary traversal, not one draft scan per range.
        let mut boundaries = text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .chain(std::iter::once(text.len()));
        let mut boundary = boundaries.next();
        for endpoint in self.spans.iter().flat_map(|s| [s.range.start, s.range.end]) {
            while boundary.is_some_and(|b| b < endpoint) {
                boundary = boundaries.next();
            }
            if boundary != Some(endpoint) {
                return Err(EditError::InvalidRange.into());
            }
        }
        Ok(())
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.spans.is_empty() && self.hint.is_none()
    }
    pub(crate) fn append_hint(&self, frame: &mut crate::presentation::Frame) {
        use crate::presentation::Run;
        if frame.source_cursor != frame.source_len {
            return;
        }
        let Some(hint) = &self.hint else {
            return;
        };
        // Never wrap or displace canonical text; reserve the terminal's last cell.
        let available = frame.columns.saturating_sub(frame.end.col + 1);
        if available < 5 {
            return;
        }
        let clipped = crate::completion::clip(&hint.text, available - 4);
        let text = format!(" [~{clipped}]");
        let width = crate::width::cells(&text);
        let row = frame.lines.last_mut().unwrap();
        row.0.extend([
            Run::Style(hint.role),
            Run::Text(text),
            Run::Style(Role::Default),
        ]);
        *frame.widths.last_mut().unwrap() += width;
        frame.end.col += width;
    }
}
/// Malformed presentation and terminal errors; staleness uses AnalysisOutcome.
#[derive(Debug)]
pub enum AnalysisPresentationError {
    /// Span count or hint size exceeds the public bound.
    Limit,
    /// Empty/reversed/overlapping or unordered spans.
    InvalidSpans,
    /// Empty hint or unsafe visible text.
    InvalidHint,
    /// Snapshot-dependent range is out of bounds or splits a grapheme.
    Edit(EditError),
    /// Invalid terminal lifecycle or I/O failure with existing cleanup context.
    Interaction(Error),
}
impl fmt::Display for AnalysisPresentationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Limit => f.write_str("analysis presentation payload limit exceeded"),
            Self::InvalidSpans => {
                f.write_str("analysis spans must be nonempty, ordered and disjoint")
            }
            Self::InvalidHint => f.write_str("invalid hint text"),
            Self::Edit(e) => e.fmt(f),
            Self::Interaction(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for AnalysisPresentationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Edit(e) => Some(e),
            Self::Interaction(e) => Some(e),
            _ => None,
        }
    }
}
impl From<EditError> for AnalysisPresentationError {
    fn from(e: EditError) -> Self {
        Self::Edit(e)
    }
}
impl From<Error> for AnalysisPresentationError {
    fn from(e: Error) -> Self {
        Self::Interaction(e)
    }
}
