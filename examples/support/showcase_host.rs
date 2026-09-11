//! Deterministic host semantics for the public-surface showcase.
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
        let mut spans = Vec::new();
        let mut token = None;
        let mut balance = 0;
        let mut invalid = None;
        for (offset, grapheme) in snapshot.text().grapheme_indices(true) {
            match grapheme {
                "{" => balance += 1,
                "}" => {
                    balance -= 1;
                    if balance < 0 && invalid.is_none() {
                        invalid = Some(offset..offset + 1);
                    }
                }
                _ => {}
            }
            if grapheme.chars().all(char::is_whitespace) {
                if let Some(begin) = token.take()
                    && spans.len() < MAX_ANALYSIS_SPANS
                {
                    spans.push(AnalysisSpan::new(begin..offset, Role::Accent).unwrap());
                }
            } else if token.is_none() {
                token = Some(offset);
            }
        }
        if let Some(begin) = token
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
        let hint = match self.snapshot.text() {
            "de" => Some(Hint::new("ploy · Tab for operations", Role::Dim)?),
            "deploy" => Some(Hint::new(" { · Enter validates", Role::Dim)?),
            _ => None,
        };
        AnalysisPresentation::new(self.snapshot.revision(), self.spans.clone(), hint)
    }

    pub fn completions(&self) -> Result<CompletionSet, CompletionError> {
        let candidates = [
            ("deploy", "Apply a local fixture plan"),
            ("describe", "Inspect fixture state"),
            ("destroy", "Show a destructive-operation preview"),
        ]
        .into_iter()
        .filter(|(value, _)| value.starts_with(self.snapshot.text()))
        .map(|(value, annotation)| {
            CompletionCandidate::new(0..self.snapshot.text().len(), value, value)?
                .with_annotation(annotation)
        })
        .collect::<Result<Vec<_>, _>>()?;
        CompletionSet::new(self.snapshot.revision(), candidates)
    }

    pub fn validation(&self) -> Result<ValidationResult, ValidationError> {
        let disposition = if let Some(range) = &self.invalid {
            ValidationDisposition::Invalid(vec![Diagnostic::new(
                "Unmatched closing brace in local plan",
                Some(range.clone()),
            )?])
        } else if self.balance > 0 {
            ValidationDisposition::Incomplete
        } else {
            ValidationDisposition::Complete
        };
        ValidationResult::new(self.snapshot.revision(), disposition)
    }
}
