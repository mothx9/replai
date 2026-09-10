//! Bounded host data and temporary selection; no discovery or scheduling.
use crate::{DraftRevision, EditError, Editor, Error, Prompt, Role};
use std::{fmt, ops::Range};
use unicode_segmentation::UnicodeSegmentation;

/// Maximum candidates admitted in one result. Order and duplicates are retained.
pub const MAX_COMPLETION_CANDIDATES: usize = 4096;
/// Maximum UTF-8 bytes in any replacement, label or annotation.
pub const MAX_COMPLETION_FIELD_BYTES: usize = 65_536;
/// Maximum combined UTF-8 payload, excluding bounded candidate metadata.
pub const MAX_COMPLETION_BYTES: usize = 4 * 1024 * 1024;

/// Invalid candidate data or a failure of the active terminal interaction.
#[derive(Debug)]
pub enum CompletionError {
    /// Candidate count, individual field or aggregate payload exceeds its limit.
    Limit,
    /// A label/annotation is empty where required or contains unsafe controls.
    InvalidDisplay,
    /// Replacement text, grapheme range or editor capacity is invalid.
    Edit(EditError),
    /// Lifecycle or terminal failure, with the ordinary cleanup contract.
    Interaction(Error),
}
impl fmt::Display for CompletionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Limit => f.write_str("completion payload limit exceeded"),
            Self::InvalidDisplay => f.write_str("invalid completion display text"),
            Self::Edit(e) => e.fmt(f),
            Self::Interaction(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for CompletionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Edit(e) => Some(e),
            Self::Interaction(e) => Some(e),
            _ => None,
        }
    }
}
impl From<EditError> for CompletionError {
    fn from(e: EditError) -> Self {
        Self::Edit(e)
    }
}
impl From<Error> for CompletionError {
    fn from(e: Error) -> Self {
        Self::Interaction(e)
    }
}
fn hidden_control(c: char) -> bool {
    matches!(c, '\u{061c}' | '\u{200b}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}' | '\u{2066}'..='\u{2069}' | '\u{feff}')
}
fn field(text: &str, display: bool) -> Result<(), CompletionError> {
    if text.len() > MAX_COMPLETION_FIELD_BYTES {
        return Err(CompletionError::Limit);
    }
    if text.chars().any(hidden_control)
        || if display {
            text.chars().any(char::is_control)
        } else {
            !crate::core::valid_text(text)
        }
    {
        return Err(if display {
            CompletionError::InvalidDisplay
        } else {
            CompletionError::Edit(EditError::InvalidText)
        });
    }
    Ok(())
}

/// One host-ordered alternative. Display and insertion are deliberately distinct.
/// Ranges are UTF-8 byte offsets in the originating snapshot, at grapheme boundaries.
/// Replacement admits LF/TAB as editor text; display fields are single-line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionCandidate {
    range: Range<usize>,
    replacement: Box<str>,
    label: Box<str>,
    annotation: Option<Box<str>>,
}
impl CompletionCandidate {
    /// Validate safe text before copying. Draft-dependent range/capacity validation
    /// occurs atomically when the entire set is installed, and again on acceptance.
    pub fn new(
        range: Range<usize>,
        replacement: &str,
        label: &str,
    ) -> Result<Self, CompletionError> {
        field(replacement, false)?;
        field(label, true)?;
        if label.is_empty() {
            return Err(CompletionError::InvalidDisplay);
        }
        if range.start > range.end {
            return Err(CompletionError::Edit(EditError::InvalidRange));
        }
        Ok(Self {
            range,
            replacement: replacement.into(),
            label: label.into(),
            annotation: None,
        })
    }
    /// Add safe optional explanation. Narrow presentation may omit or ellipsize it.
    pub fn with_annotation(mut self, annotation: &str) -> Result<Self, CompletionError> {
        field(annotation, true)?;
        self.annotation = Some(annotation.into());
        Ok(self)
    }
    /// Replacement range in the originating snapshot.
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
    /// Exact insertion, never shortened by presentation.
    pub fn replacement(&self) -> &str {
        &self.replacement
    }
    /// Safe display label, independent of insertion.
    pub fn label(&self) -> &str {
        &self.label
    }
    /// Optional safe explanation.
    pub fn annotation(&self) -> Option<&str> {
        self.annotation.as_deref()
    }
    fn bytes(&self) -> usize {
        self.replacement.len() + self.label.len() + self.annotation.as_ref().map_or(0, |s| s.len())
    }
}

/// An immutable bounded result for one retained editor's draft revision.
/// Cloning copies the bounded candidate payload. Retain/share it in the host if needed.
/// No request ordering is inferred: explicit deliveries at the same revision replace
/// the previous presentation in delivery order. The host filters superseded jobs.
#[derive(Clone, Debug)]
pub struct CompletionSet {
    revision: DraftRevision,
    candidates: Box<[CompletionCandidate]>,
}
impl CompletionSet {
    /// Admit bounded data without sorting, deduplicating or inspecting meaning.
    /// Empty sets silently dismiss; every nonempty set requires explicit acceptance.
    pub fn new(
        revision: DraftRevision,
        candidates: Vec<CompletionCandidate>,
    ) -> Result<Self, CompletionError> {
        if candidates.len() > MAX_COMPLETION_CANDIDATES
            || candidates
                .iter()
                .map(CompletionCandidate::bytes)
                .sum::<usize>()
                > MAX_COMPLETION_BYTES
        {
            return Err(CompletionError::Limit);
        }
        Ok(Self {
            revision,
            candidates: candidates.into_boxed_slice(),
        })
    }
    /// Originating draft identity, not host-context or discovery-job identity.
    pub fn revision(&self) -> DraftRevision {
        self.revision
    }
    /// Host order, including duplicates.
    pub fn candidates(&self) -> &[CompletionCandidate] {
        &self.candidates
    }
    pub(crate) fn validate(&self, editor: &Editor) -> Result<(), CompletionError> {
        for c in &self.candidates {
            editor.validate_replacement(&c.range, &c.replacement)?;
        }
        Ok(())
    }
}

/// A serialized host selection operation. These never perform candidate discovery.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionAction {
    /// Next candidate, wrapping in host order. Draft unchanged.
    Next,
    /// Previous candidate, wrapping in host order. Draft unchanged.
    Previous,
    /// Atomically apply the selected candidate if its draft is still current.
    Accept,
    /// Remove temporary presentation; draft unchanged.
    Dismiss,
}
/// Inspectable selection, with no renderer or terminal ownership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompletionSelection {
    /// Originating draft revision.
    pub revision: DraftRevision,
    /// Zero-based selected candidate in the original host order.
    pub index: usize,
    /// Number of candidates, including structural duplicates.
    pub count: usize,
}
pub(crate) struct ActiveCompletion {
    pub set: CompletionSet,
    pub selected: usize,
}
impl ActiveCompletion {
    pub fn selection(&self) -> CompletionSelection {
        CompletionSelection {
            revision: self.set.revision,
            index: self.selected,
            count: self.set.candidates.len(),
        }
    }
    pub fn candidate(&self) -> &CompletionCandidate {
        &self.set.candidates[self.selected]
    }
    pub fn navigate(&mut self, previous: bool) {
        let count = self.set.candidates.len();
        self.selected = if previous {
            (self.selected + count - 1) % count
        } else {
            (self.selected + 1) % count
        };
    }
    /// One frame and one existing transition renderer, including the editable cursor.
    pub fn frame(
        &self,
        editor: &Editor,
        prompt: &Prompt,
        size: (usize, usize),
    ) -> crate::presentation::Frame {
        use crate::presentation::{Frame, Line, Point, Run};
        let height = size
            .1
            .saturating_sub(2)
            .clamp(1, 8)
            .min(self.set.candidates.len() + 1);
        let header = usize::from(height > 1);
        let visible = height - header;
        let first = self.selected / visible * visible;
        let end = (first + visible).min(self.set.candidates.len());
        let actual_height = end - first + header;
        let mut frame = Frame::new(
            editor,
            prompt,
            size.0,
            size.1.saturating_sub(actual_height).max(2),
        );
        let mut append = |runs: Vec<Run>, width: usize| {
            frame.lines.push(Line(runs));
            frame.widths.push(width);
        };
        if header != 0 {
            let text = clip(
                &format!(
                    "{} / {}  Tab next · Enter accept · Esc dismiss",
                    self.selected + 1,
                    self.set.candidates.len()
                ),
                size.0.saturating_sub(1),
            );
            let width = crate::width::cells(&text);
            append(
                vec![
                    Run::Style(Role::Dim),
                    Run::Text(text),
                    Run::Style(Role::Default),
                ],
                width,
            );
        }
        let widest_label = self.set.candidates[first..end]
            .iter()
            .map(|c| crate::width::cells(c.label()))
            .max()
            .unwrap_or(0);
        for index in first..end {
            let c = &self.set.candidates[index];
            let room = size.0.saturating_sub(3);
            let annotation = c.annotation().filter(|a| !a.is_empty() && size.0 >= 40);
            let label_room = if annotation.is_some() {
                widest_label.min(room / 2)
            } else {
                room
            };
            let label = clip(c.label(), label_room);
            let label_width = crate::width::cells(&label);
            let mut runs = vec![
                Run::Style(if index == self.selected {
                    Role::Accent
                } else {
                    Role::Default
                }),
                Run::Text(format!(
                    "{}{}",
                    if index == self.selected { "> " } else { "  " },
                    label
                )),
            ];
            let mut width = 2 + label_width;
            if let Some(annotation) = annotation {
                let gap = label_room - label_width + 2;
                let text = clip(annotation, room.saturating_sub(label_room + 2));
                width += gap + crate::width::cells(&text);
                runs.extend([
                    Run::Style(Role::Dim),
                    Run::Text(format!("{}{text}", " ".repeat(gap))),
                ]);
            }
            runs.push(Run::Style(Role::Default));
            append(runs, width);
        }
        frame.rows = size.1;
        frame.end = Point {
            row: frame.lines.len() - 1,
            col: *frame.widths.last().unwrap(),
        };
        frame
    }
}
fn clip(text: &str, columns: usize) -> String {
    if crate::width::cells(text) <= columns {
        return text.into();
    }
    if columns == 0 {
        return String::new();
    }
    let mut result = String::new();
    let mut width = 0;
    for grapheme in text.graphemes(true) {
        let cells = crate::width::cells(grapheme);
        if width + cells >= columns {
            break;
        }
        result.push_str(grapheme);
        width += cells;
    }
    result.push('…');
    result
}
