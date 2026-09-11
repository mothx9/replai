//! Deterministic public-API showcase. No command, deployment or service is used.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[path = "support/showcase_host.rs"]
mod host;

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn prompt() -> Result<replai::Prompt, replai::EditError> {
    use replai::{Prompt, Role, Span, Text};
    Prompt::composed(Text::from_spans(vec![
        Span::new(Role::Accent, "ops")?,
        Span::new(Role::Dim, "[local]")?,
        Span::new(Role::Default, "> ")?,
    ])?)?
    .with_continuation_text(Text::from_spans(vec![
        Span::new(Role::Dim, "    ")?,
        Span::new(Role::Accent, "... ")?,
    ])?)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn result_document(text: &str) -> Result<replai::Document, replai::EditError> {
    use replai::{Block, Document, Role, Severity, Text};
    Document::new(vec![
        Block::Heading {
            level: 2,
            text: Text::new("Local fixture result")?,
        },
        Block::KeyValue(vec![
            (
                Text::new("Input bytes")?,
                Text::new(&text.len().to_string())?,
            ),
            (
                Text::new("Execution")?,
                Text::styled(Role::Dim, "not performed")?,
            ),
        ]),
        Block::Status {
            severity: Severity::Success,
            text: Text::new("Host accepted the validated draft")?,
        },
    ])
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use replai::*;
    use std::io::IsTerminal;
    use std::time::{Duration, Instant};

    let mut input = Interaction::new(Editor::new(65_536, 100));
    input.set_submission_policy(SubmissionPolicy::Validated)?;
    println!("REPLAI public API showcase · deterministic local fixture");
    println!("Type de + Tab · braces continue · Up recalls · Ctrl-D exits\n");

    loop {
        input.open_driven(&std::io::stdin(), &std::io::stdout(), prompt()?)?;
        let mut parsed = None;
        let mut revision = None;
        let mut notice_at = None;
        let mut notice_sent = false;
        loop {
            let mut event = input.poll(Duration::from_millis(25))?;
            if input.is_open() && revision != Some(input.revision()) {
                let current = host::Parsed::new(input.analysis_snapshot());
                input.present_analysis(current.presentation()?)?;
                parsed = Some(current);
                revision = Some(input.revision());
            }
            match event {
                Some(Event::CompletionRequested) => {
                    input.present_completions(parsed.as_ref().unwrap().completions()?)?;
                    if !notice_sent {
                        notice_at.get_or_insert(Instant::now() + Duration::from_millis(700));
                    }
                }
                Some(Event::SubmissionRequested(_)) => {
                    event = input
                        .apply_validation(parsed.as_ref().unwrap().validation()?)?
                        .event;
                }
                _ => {}
            }
            if input.is_open() && notice_at.is_some_and(|due| Instant::now() >= due) {
                let before = input.analysis_snapshot();
                let selected = input.completion_selection();
                input.external_output(
                    Role::Dim,
                    "[host] fixture inventory refreshed · 3 operations available",
                )?;
                assert_eq!(before.revision(), input.revision());
                assert_eq!(before.text(), input.editor().text());
                assert_eq!(before.cursor(), input.editor().cursor());
                assert_eq!(selected, input.completion_selection());
                notice_at = None;
                notice_sent = true;
            }
            match event {
                Some(Event::Submitted(text)) => {
                    let document = result_document(&text)?;
                    document.write_to(
                        &mut std::io::stdout().lock(),
                        84,
                        Theme::from_environment(std::io::stdout().is_terminal()),
                    )?;
                    if !text.is_empty() {
                        input.editor_mut()?.admit_history(&text)?;
                    }
                    input.editor_mut()?.clear();
                    println!();
                    break;
                }
                Some(Event::Interrupted) => {
                    input.editor_mut()?.clear();
                    break;
                }
                Some(Event::EndOfInput) => return Ok(()),
                Some(Event::Rejected(error)) => {
                    input.external_output(Role::Warning, &error.to_string())?;
                }
                _ => {}
            }
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn main() {
    eprintln!("This showcase requires the Linux/macOS terminal backend.");
}
