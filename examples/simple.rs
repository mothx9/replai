//! A retained reader; evaluation and history admission belong to this host.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use replai::{Editor, Interaction, Prompt, ReadOutcome};
    let mut reader = Interaction::new(Editor::new(65_536, 100));
    loop {
        match reader.read_line(Prompt::new("simple")?)? {
            ReadOutcome::Submitted(line) => {
                println!("echo: {line}");
                if line == "/exit" {
                    break;
                }
                if !line.is_empty() {
                    reader.editor_mut()?.admit_history(&line)?;
                }
                reader.editor_mut()?.clear();
            }
            ReadOutcome::Interrupted => reader.editor_mut()?.clear(),
            ReadOutcome::EndOfInput => break,
        }
    }
    Ok(())
}
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn main() {
    eprintln!("This example requires a qualified Linux/macOS terminal backend.");
}
