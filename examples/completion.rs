//! The host discovers and orders; REPLAI presents, navigates and accepts.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use replai::{CompletionCandidate, CompletionSet, Editor, Event, Interaction, Prompt};
    use std::time::Duration;
    let mut interaction = Interaction::new(Editor::new(4096, 100));
    loop {
        interaction.open(&std::io::stdin(), &std::io::stdout(), Prompt::new("demo")?)?;
        loop {
            match interaction.poll(Duration::from_secs(1))? {
                Some(Event::CompletionRequested) => {
                    let snapshot = interaction.analysis_snapshot();
                    // This catalog, prefix filter and order belong entirely to this host.
                    let candidates = [
                        ("build", "Build the project"),
                        ("bundle", "Produce a distributable bundle"),
                        ("burn", "Run a stress exercise"),
                        ("check", "Check project configuration"),
                    ]
                    .into_iter()
                    .filter(|(word, _)| word.starts_with(snapshot.text()))
                    .map(|(word, description)| {
                        CompletionCandidate::new(0..snapshot.text().len(), word, word)?
                            .with_annotation(description)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                    interaction.present_completions(CompletionSet::new(
                        snapshot.revision(),
                        candidates,
                    )?)?;
                }
                Some(Event::Submitted(text)) => {
                    println!("host received: {text}");
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
                Some(Event::Rejected(error)) => {
                    interaction.external_output(replai::Role::Warning, &error.to_string())?
                }
                Some(Event::SubmissionRequested(_)) => unreachable!("direct submission"),
                None => {}
            }
        }
    }
}
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn main() {
    eprintln!("This example requires the Linux/macOS terminal backend.");
}
