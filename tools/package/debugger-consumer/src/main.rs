//! Independent deterministic debugger-style host used only as a packaged consumer.
use replai::*;

#[derive(Clone)]
struct DebuggerAnalysis {
    snapshot: AnalysisSnapshot,
}

impl DebuggerAnalysis {
    fn new(snapshot: AnalysisSnapshot) -> Self {
        Self { snapshot }
    }

    fn presentation(&self) -> Result<AnalysisPresentation, AnalysisPresentationError> {
        let end = self
            .snapshot
            .text()
            .find(char::is_whitespace)
            .unwrap_or(self.snapshot.text().len());
        let spans = (end > 0)
            .then(|| AnalysisSpan::new(0..end, Role::Accent))
            .transpose()?
            .into_iter()
            .collect();
        let hint = match self.snapshot.text() {
            "br" => Some(Hint::new("eak main.rs:42", Role::Dim)?),
            "fra" => Some(Hint::new("me 0", Role::Dim)?),
            _ => None,
        };
        AnalysisPresentation::new(self.snapshot.revision(), spans, hint)
    }

    fn completions(&self) -> Result<CompletionSet, CompletionError> {
        let prefix = self.snapshot.text();
        let items = [
            ("break main.rs:42", "Set fixture breakpoint"),
            ("backtrace", "List fixture frames"),
            ("continue", "Advance fixture target"),
            ("frame 0", "Select fixture frame"),
            ("print counter", "Inspect fixture expression"),
            ("watch {", "Begin multiline watch"),
        ];
        let candidates = items
            .into_iter()
            .filter(|(value, _)| value.starts_with(prefix))
            .map(|(value, annotation)| {
                CompletionCandidate::new(0..prefix.len(), value, value)?
                    .with_annotation(annotation)
            })
            .collect::<Result<Vec<_>, _>>()?;
        CompletionSet::new(self.snapshot.revision(), candidates)
    }

    fn validation(&self) -> Result<ValidationResult, ValidationError> {
        let mut balance = 0i32;
        let mut invalid = None;
        for (index, byte) in self.snapshot.text().bytes().enumerate() {
            match byte {
                b'{' => balance += 1,
                b'}' => {
                    balance -= 1;
                    if balance < 0 && invalid.is_none() {
                        invalid = Some(index..index + 1);
                    }
                }
                _ => {}
            }
        }
        let disposition = if let Some(range) = invalid {
            ValidationDisposition::Invalid(vec![Diagnostic::new(
                "unmatched closing brace in watch expression",
                Some(range),
            )?])
        } else if balance > 0 {
            ValidationDisposition::Incomplete
        } else {
            ValidationDisposition::Complete
        };
        ValidationResult::new(self.snapshot.revision(), disposition)
    }
}

fn result_document(text: &str) -> Result<Document, EditError> {
    Document::new(vec![
        Block::Heading {
            level: 2,
            text: Text::new("Fixture debugger result")?,
        },
        Block::KeyValue(vec![
            (Text::new("target")?, Text::new("demo-process")?),
            (Text::new("frame")?, Text::new("0 · main.rs:42")?),
            (Text::new("request bytes")?, Text::new(&text.len().to_string())?),
        ]),
        Block::Status {
            severity: Severity::Success,
            text: Text::new("host accepted the validated command")?,
        },
    ])
}

fn portable_contract() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = Interaction::new(Editor::new(65_536, 32));
    input.editor_mut()?.insert("br")?;
    let stale_snapshot = input.analysis_snapshot();
    input.editor_mut()?.insert("e")?;
    let stale = DebuggerAnalysis::new(stale_snapshot);
    assert_eq!(input.present_analysis(stale.presentation()?)?, AnalysisOutcome::Stale);
    assert_eq!(input.present_completions(stale.completions()?)?, AnalysisOutcome::Stale);
    assert_eq!(input.editor().text(), "bre");
    assert!(input.analysis_presentation().is_none());
    assert!(input.completion_selection().is_none());
    eprintln!("ASSERT stale-analysis-refused");
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn native() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::{Duration, Instant};
    portable_contract()?;
    let mut input = Interaction::new(Editor::new(65_536, 32));
    input.set_submission_policy(SubmissionPolicy::Validated)?;
    println!("Packaged debugger fixture · deterministic · no process attached");
    input.open_driven(
        &std::io::stdin(),
        &std::io::stdout(),
        Prompt::new("debug")?.with_continuation("  ...")?,
    )?;
    let mut parsed = DebuggerAnalysis::new(input.analysis_snapshot());
    let mut revision = None;
    let mut delayed: Option<AnalysisPresentation> = None;
    let mut delayed_once = false;
    let mut notices: Vec<Instant> = Vec::new();
    loop {
        let mut event = input.poll(Duration::from_millis(20))?;
        if input.is_open() && revision != Some(input.revision()) {
            let current = DebuggerAnalysis::new(input.analysis_snapshot());
            if let Some(old) = delayed.take() {
                assert_eq!(input.present_analysis(old)?, AnalysisOutcome::Stale);
                eprintln!("ASSERT delayed-stale-refused");
            }
            if !current.snapshot.text().is_empty() && !delayed_once {
                delayed = Some(current.presentation()?);
                delayed_once = true;
                eprintln!("ASSERT delayed-analysis-captured");
            } else {
                assert_eq!(input.present_analysis(current.presentation()?)?, AnalysisOutcome::Applied);
                eprintln!("ASSERT fresh-analysis-applied");
            }
            parsed = current;
            revision = Some(input.revision());
        }
        match event {
            Some(Event::CompletionRequested) => {
                assert_eq!(input.present_completions(parsed.completions()?)?, AnalysisOutcome::Applied);
                notices = vec![
                    Instant::now() + Duration::from_millis(80),
                    Instant::now() + Duration::from_millis(160),
                ];
                eprintln!("ASSERT completion-presented");
            }
            Some(Event::SubmissionRequested(_)) => {
                let decision = parsed.validation()?;
                eprintln!("ASSERT disposition-{:?}", decision.disposition());
                let result = input.apply_validation(decision)?;
                eprintln!("ASSERT validation-{:?}", result.analysis);
                event = result.event;
            }
            _ => {}
        }
        if notices.first().is_some_and(|due| Instant::now() >= *due) {
            let before = input.analysis_snapshot();
            let selection = input.completion_selection();
            let index = 3 - notices.len();
            input.external_output(Role::Dim, &format!("[debug-event] module batch {index}/2 loaded"))?;
            assert_eq!(before.revision(), input.revision());
            assert_eq!(before.text(), input.editor().text());
            assert_eq!(before.cursor(), input.editor().cursor());
            assert_eq!(selection, input.completion_selection());
            notices.remove(0);
            eprintln!("ASSERT serialized-notice-preserved");
        }
        match event {
            Some(Event::Submitted(text)) => {
                eprintln!("ASSERT submitted {}", text.escape_default());
                input.output_document(&result_document(&text)?)?;
                input.close()?;
                return Ok(());
            }
            Some(Event::Interrupted | Event::EndOfInput) => {
                input.close()?;
                return Ok(());
            }
            Some(Event::Rejected(error)) => input.external_output(Role::Warning, &error.to_string())?,
            _ => {}
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().any(|argument| argument == "--portable-check") {
        portable_contract()
    } else {
        native()
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    portable_contract()
}
