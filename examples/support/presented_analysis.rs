//! One example-owned parse yields three independent revision-bound products.
use replai::*;
use unicode_segmentation::UnicodeSegmentation;
pub struct Parsed {
    snapshot: AnalysisSnapshot,
    spans: Vec<AnalysisSpan>,
    balance: i32,
    invalid: Option<std::ops::Range<usize>>,
}
impl Parsed {
    pub fn new(snapshot: AnalysisSnapshot) -> Self {
        let mut balance = 0;
        let mut invalid = None;
        let mut spans = Vec::new();
        let mut start = None;
        for (offset, g) in snapshot.text().grapheme_indices(true) {
            if g == "{" {
                balance += 1;
            }
            if g == "}" {
                balance -= 1;
                if balance < 0 && invalid.is_none() {
                    invalid = Some(offset..offset + 1);
                }
            }
            if g.chars().all(char::is_whitespace) {
                if let Some(begin) = start.take()
                    && spans.len() < MAX_ANALYSIS_SPANS
                {
                    spans.push(AnalysisSpan::new(begin..offset, Role::Accent).unwrap());
                }
            } else if start.is_none() {
                start = Some(offset);
            }
        }
        if let Some(begin) = start
            && spans.len() < MAX_ANALYSIS_SPANS
        {
            spans.push(AnalysisSpan::new(begin..snapshot.text().len(), Role::Accent).unwrap());
        }
        Self {
            snapshot,
            spans,
            balance,
            invalid,
        }
    }
    pub fn presentation(&self) -> Result<AnalysisPresentation, AnalysisPresentationError> {
        let hint = Hint::new(
            if self.snapshot.text() == "bu" {
                "ild · Tab for candidates"
            } else {
                "host analysis · Enter validates"
            },
            Role::Dim,
        )?;
        AnalysisPresentation::new(self.snapshot.revision(), self.spans.clone(), Some(hint))
    }
    pub fn completions(&self) -> Result<CompletionSet, CompletionError> {
        let candidates = [
            ("build", "Build the project"),
            ("bundle", "Produce a bundle"),
            ("burn", "Example third candidate"),
        ]
        .into_iter()
        .filter(|(word, _)| word.starts_with(self.snapshot.text()))
        .map(|(word, label)| {
            CompletionCandidate::new(0..self.snapshot.text().len(), word, word)?
                .with_annotation(label)
        })
        .collect::<Result<Vec<_>, _>>()?;
        CompletionSet::new(self.snapshot.revision(), candidates)
    }
    pub fn validation(&self) -> Result<ValidationResult, ValidationError> {
        let d = if let Some(r) = &self.invalid {
            ValidationDisposition::Invalid(vec![Diagnostic::new(
                "Unmatched closing brace",
                Some(r.clone()),
            )?])
        } else if self.balance > 0 {
            ValidationDisposition::Incomplete
        } else {
            ValidationDisposition::Complete
        };
        ValidationResult::new(self.snapshot.revision(), d)
    }
}
