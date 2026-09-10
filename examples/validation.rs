//! A tiny host grammar; no language logic enters REPLAI.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[path = "support/validation.rs"]
mod validator;
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use replai::{Editor, Event, Interaction, Prompt, SubmissionPolicy, ValidationResult};
    use std::time::Duration;
    let mut interaction = Interaction::new(Editor::new(1_048_576, 100));
    interaction.set_submission_policy(SubmissionPolicy::Validated)?;
    println!(
        "Host fixture: balanced braces submit; an open brace continues; unmatched close is invalid. Ctrl-D on empty exits."
    );
    loop {
        interaction.open_driven(
            &std::io::stdin(),
            &std::io::stdout(),
            Prompt::new("validate")?,
        )?;
        loop {
            let mut event = interaction.poll(Duration::from_secs(1))?;
            if let Some(Event::SubmissionRequested(snapshot)) = event {
                let disposition = validator::classify(&snapshot)?;
                event = interaction
                    .apply_validation(ValidationResult::new(snapshot.revision(), disposition)?)?
                    .event;
            }
            match event {
                Some(Event::Submitted(text)) => {
                    println!("host received {} bytes:\n{text}", text.len());
                    if !text.is_empty() {
                        interaction.editor_mut()?.admit_history(&text)?;
                    }
                    interaction.editor_mut()?.clear();
                    break;
                }
                Some(Event::Interrupted) => {
                    interaction.editor_mut()?.clear();
                    break;
                }
                Some(Event::EndOfInput) => return Ok(()),
                Some(Event::Rejected(e)) => {
                    interaction.external_output(replai::Role::Warning, &e.to_string())?
                }
                Some(Event::CompletionRequested) | None => {}
                Some(Event::SubmissionRequested(_)) => unreachable!("handled above"),
            }
        }
    }
}
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn main() {
    eprintln!("This example requires the Linux/macOS terminal backend.");
}
