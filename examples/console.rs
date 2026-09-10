//! A local host fixture: one analysis, completion, validation and timed output.
//! No command is executed and no service is contacted.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[path = "support/presented_analysis.rs"]
mod host;

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use replai::*;
    use std::time::{Duration, Instant};

    let mut input = Interaction::new(Editor::new(65_536, 100));
    input.set_submission_policy(SubmissionPolicy::Validated)?;
    println!("Local host fixture / no commands executed, no service required.");
    println!("Tab chooses; Enter accepts; braces continue; Ctrl-D on empty exits.\n");
    loop {
        input.open_driven(&std::io::stdin(), &std::io::stdout(), Prompt::new("demo")?)?;
        let mut parsed = None;
        let mut revision = None;
        let mut notice_at = None;
        let mut notice_sent = false;
        loop {
            let mut event = input.poll(Duration::from_millis(50))?;
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
                        notice_at.get_or_insert(Instant::now() + Duration::from_secs(2));
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
                    "[host] Local result: 3 build targets available.",
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
                    println!(
                        "[ok] host received {} bytes; nothing executed.\n",
                        text.len()
                    );
                    if !text.is_empty() {
                        input.editor_mut()?.admit_history(&text)?;
                    }
                    input.editor_mut()?.clear();
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
    eprintln!("This example requires the Linux/macOS terminal backend.");
}
