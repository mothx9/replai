//! Host-owned submission meaning, bounded revision-bound presentation.
use crate::{AnalysisOutcome, DraftRevision, EditError, Error};
use std::{fmt, ops::Range};

/// Maximum diagnostics retained for one invalid submission.
pub const MAX_DIAGNOSTICS: usize = 32;
/// Maximum UTF-8 bytes per safe single-line diagnostic message.
pub const MAX_DIAGNOSTIC_BYTES: usize = 4096;
/// Maximum aggregate diagnostic message bytes, excluding bounded metadata.
pub const MAX_VALIDATION_BYTES: usize = 65_536;

/// Submission and multiline key policy. Configure while the interaction is closed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SubmissionPolicy {
    /// Enter submits directly; Up/Down retain the existing history bindings.
    #[default]
    Direct,
    /// Enter asks the host; Up/Down move between logical lines, then history at edges.
    /// On continuation lines, Tab in the leading ASCII space/tab prefix inserts
    /// spaces to the next four-cell stop relative to the logical line. Elsewhere
    /// Tab requests completion; an active completion menu always takes precedence.
    Validated,
}
/// Safe host explanation of invalid input. No parser, fix-it or highlighting semantics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    message: Box<str>,
    range: Option<Range<usize>>,
}
impl Diagnostic {
    /// Validate before copying. Messages are nonempty single-line safe text;
    /// draft-dependent grapheme ranges are checked atomically on result delivery.
    pub fn new(message: &str, range: Option<Range<usize>>) -> Result<Self, ValidationError> {
        if message.len() > MAX_DIAGNOSTIC_BYTES {
            return Err(ValidationError::Limit);
        }
        if message.is_empty()
            || message
                .chars()
                .any(|c| c.is_control() || crate::completion::hidden_control(c))
        {
            return Err(ValidationError::InvalidMessage);
        }
        if range.as_ref().is_some_and(|r| r.start > r.end) {
            return Err(EditError::InvalidRange.into());
        }
        Ok(Self {
            message: message.into(),
            range,
        })
    }
    /// Complete message, even when its temporary display is ellipsized.
    pub fn message(&self) -> &str {
        &self.message
    }
    /// Optional byte range in the originating snapshot, never terminal coordinates.
    pub fn range(&self) -> Option<Range<usize>> {
        self.range.clone()
    }
}
/// A host semantic decision, not an editor error or a terminal capability.
#[derive(Clone, Debug)]
pub enum ValidationDisposition {
    /// Submit exactly the requested current draft once and restore the terminal.
    Complete,
    /// Insert LF at the current cursor atomically and continue editing.
    Incomplete,
    /// Preserve the draft and present bounded explanations. Empty explanations are valid.
    Invalid(Vec<Diagnostic>),
}
/// An immutable bounded decision for the originating retained editor's revision.
#[derive(Clone, Debug)]
pub struct ValidationResult {
    revision: DraftRevision,
    disposition: ValidationDisposition,
}
impl ValidationResult {
    /// Check resource bounds before admission. Delivery also requires a pending
    /// Enter request at this revision; this value cannot force unsolicited submission.
    pub fn new(
        revision: DraftRevision,
        disposition: ValidationDisposition,
    ) -> Result<Self, ValidationError> {
        if let ValidationDisposition::Invalid(diagnostics) = &disposition
            && (diagnostics.len() > MAX_DIAGNOSTICS
                || diagnostics.iter().map(|d| d.message.len()).sum::<usize>()
                    > MAX_VALIDATION_BYTES)
        {
            return Err(ValidationError::Limit);
        }
        Ok(Self {
            revision,
            disposition,
        })
    }
    /// Exact draft provenance; not an application context or job identity.
    pub fn revision(&self) -> DraftRevision {
        self.revision
    }
    /// Host classification and bounded explanation.
    pub fn disposition(&self) -> &ValidationDisposition {
        &self.disposition
    }
    pub(crate) fn into_disposition(self) -> ValidationDisposition {
        self.disposition
    }
}
/// Result of serialized delivery. Submission carries the ordinary restored-terminal event.
#[derive(Debug, PartialEq)]
pub struct ValidationOutcome {
    /// Applied or stale. Stale delivery emits no terminal mutations or event.
    pub analysis: AnalysisOutcome,
    /// Submitted after Complete, otherwise None. Never submit a second time by polling.
    pub event: Option<crate::Event>,
}
/// Malformed host data or lifecycle/I/O failure, distinct from semantic Invalid.
#[derive(Debug)]
pub enum ValidationError {
    /// Count, message or total payload exceeds the documented limit.
    Limit,
    /// Empty or unsafe diagnostic text.
    InvalidMessage,
    /// No pending Enter request at this revision (including a consumed response).
    NoRequest,
    /// Invalid range or insufficient capacity for continued input; unchanged draft.
    Edit(EditError),
    /// Terminal/lifecycle failure, including cleanup context.
    Interaction(Error),
}
impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Limit => f.write_str("validation payload limit exceeded"),
            Self::InvalidMessage => f.write_str("invalid diagnostic message"),
            Self::NoRequest => f.write_str("no pending submission request at this revision"),
            Self::Edit(e) => e.fmt(f),
            Self::Interaction(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for ValidationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Edit(e) => Some(e),
            Self::Interaction(e) => Some(e),
            _ => None,
        }
    }
}
impl From<EditError> for ValidationError {
    fn from(e: EditError) -> Self {
        Self::Edit(e)
    }
}
impl From<Error> for ValidationError {
    fn from(e: Error) -> Self {
        Self::Interaction(e)
    }
}

#[derive(Default)]
pub(crate) struct ValidationState {
    pub pending: Option<DraftRevision>,
    pub diagnostics: Option<Vec<Diagnostic>>,
}
impl ValidationState {
    /// Bounded temporary rows use the same frame, width policy and renderer as editing.
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
        let diagnostics = self.diagnostics.as_ref().unwrap();
        let height = (diagnostics.len() + 1)
            .min(5)
            .min(size.1.saturating_sub(2).max(1));
        let mut frame = Frame::analyzed(
            editor,
            prompt,
            size.0,
            size.1.saturating_sub(height).max(2),
            analysis,
        );
        if let Some(a) = analysis {
            a.append_hint(&mut frame);
        }
        let header = format!("! Invalid input · {} diagnostics", diagnostics.len());
        for i in 0..height {
            let text = if i == 0 {
                header.clone()
            } else {
                let d = &diagnostics[i - 1];
                match &d.range {
                    Some(r) => format!("! [{}..{}] {}", r.start, r.end, d.message),
                    None => format!("! {}", d.message),
                }
            };
            let text = crate::completion::clip(&text, size.0.saturating_sub(1));
            frame.widths.push(crate::width::cells(&text));
            frame.lines.push(Line(vec![
                Run::Style(Role::Error),
                Run::Text(text),
                Run::Style(Role::Default),
            ]));
        }
        frame.rows = size.1;
        frame.end = Point {
            row: frame.lines.len() - 1,
            col: *frame.widths.last().unwrap(),
        };
        frame
    }
}
