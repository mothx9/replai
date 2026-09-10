//! Session host: one immutable snapshot, one example parse, three derived products.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[path = "support/presented_analysis.rs"]
mod host;
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use replai::*;
    use std::time::Duration;
    let mut input = Interaction::new(Editor::new(1_048_576, 100));
    input.set_submission_policy(SubmissionPolicy::Validated)?;
    println!(
        "Host analysis demo: type bu, Tab to choose; braces enable multiline. [~...] is a hint, never submitted. Ctrl-D on empty exits."
    );
    loop {
        input.open_driven(
            &std::io::stdin(),
            &std::io::stdout(),
            Prompt::new("analyze")?,
        )?;
        let mut parsed = None;
        let mut revision = None;
        loop {
            let mut event = input.poll(Duration::from_millis(100))?;
            if input.is_open() && revision != Some(input.revision()) {
                let current = host::Parsed::new(input.analysis_snapshot());
                input.present_analysis(current.presentation()?)?;
                revision = Some(input.revision());
                parsed = Some(current);
            }
            match event {
                Some(Event::CompletionRequested) => {
                    input.present_completions(parsed.as_ref().unwrap().completions()?)?;
                }
                Some(Event::SubmissionRequested(_)) => {
                    event = input
                        .apply_validation(parsed.as_ref().unwrap().validation()?)?
                        .event;
                }
                _ => {}
            }
            match event {
                Some(Event::Submitted(text)) => {
                    println!("host received: {text}");
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
                Some(Event::Rejected(e)) => {
                    input.external_output(Role::Warning, &e.to_string())?;
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
