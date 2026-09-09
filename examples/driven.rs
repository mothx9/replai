//! A host reactor: terminal readiness, one application timer and optional socket events.
//! No REPLAI compatibility poll is called. The host owns every wait and notification.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[path = "support/wait.rs"]
mod wait;
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use replai::{
        Block, Document, Editor, Event, Interaction, Prompt, Role, Text, WaitInterest, Wake,
    };
    use std::{
        io::Read,
        os::{fd::AsFd, unix::net::UnixStream},
        time::{Duration, Instant},
    };
    let mut app = std::env::args()
        .nth(1)
        .map(UnixStream::connect)
        .transpose()?;
    let mut interaction = Interaction::new(Editor::new(65_536, 100));
    let mut timer = app
        .is_none()
        .then(|| Instant::now() + Duration::from_secs(2));
    interaction.open_driven(
        &std::io::stdin(),
        &std::io::stdout(),
        Prompt::new("driven")?,
    )?;
    eprintln!(
        "SIZES {} {} {} {}",
        std::mem::size_of::<Interaction>(),
        std::mem::size_of::<replai::Deadline>(),
        std::mem::size_of::<WaitInterest>(),
        std::mem::size_of::<Wake>()
    );
    eprintln!("READY");
    let mut advances = 0;
    loop {
        let interest = interaction
            .is_open()
            .then(|| interaction.wait_interest())
            .transpose()?;
        let deadline = match interest {
            Some(WaitInterest::Input { deadline }) => deadline,
            _ => None,
        };
        let due = deadline.map(|d| d.at()).into_iter().chain(timer).min();
        let timeout = if interest == Some(WaitInterest::Ready) {
            Some(Duration::ZERO)
        } else {
            due.map(|d| d.saturating_duration_since(Instant::now()))
        };
        // Borrows end with this wait. No reactor registration survives close/reopen.
        let (input_ready, app_ready) = {
            let mut sources = Vec::new();
            if interaction.is_open() {
                sources.push(interaction.input_source()?);
            }
            if let Some(app) = &app {
                sources.push(app.as_fd());
            }
            let ready = wait::wait(&sources, timeout)?;
            (
                interaction.is_open() && ready[0],
                app.is_some() && ready.last().copied().unwrap_or(false),
            )
        };
        let mut outcome = None;
        if input_ready || interest == Some(WaitInterest::Ready) {
            let started = Instant::now();
            outcome = interaction.advance(Wake::InputReady)?;
            let elapsed = started.elapsed();
            if app.is_some() {
                eprintln!("READY_ADVANCE_NS {}", elapsed.as_nanos());
            }
            advances += 1;
        } else if let Some(token) = deadline.filter(|d| Instant::now() >= d.at()) {
            outcome = interaction.advance(Wake::Deadline(token))?;
            advances += 1;
            eprintln!("DEADLINE");
        }
        let mut notice = timer.is_some_and(|t| Instant::now() >= t);
        if notice {
            timer = None;
        }
        if app_ready {
            let mut command = [0];
            if app.as_mut().unwrap().read(&mut command)? == 0 {
                break;
            }
            match command[0] {
                b'O' => notice = true,
                b'R' => {
                    interaction.advance(Wake::Resize)?;
                    advances += 1;
                    eprintln!("RESIZED");
                }
                b'X' => {
                    outcome = Some(interaction.interrupt()?);
                }
                b'Q' => {
                    interaction.close()?;
                    eprintln!("CLOSED");
                }
                b'N' => {
                    interaction.editor_mut()?.clear();
                    interaction.open_driven(
                        &std::io::stdin(),
                        &std::io::stdout(),
                        Prompt::new("driven")?,
                    )?;
                    eprintln!("READY");
                }
                b'E' => {
                    interaction.close()?;
                    break;
                }
                b'S' => eprintln!("OBSERVER"),
                _ => return Err("unknown example application event".into()),
            }
        }
        if notice && interaction.is_open() {
            interaction.output_document(&Document::new(vec![Block::Paragraph(Text::new(
                "application event: draft preserved",
            )?)])?)?;
            eprintln!("OUTPUT");
        }
        if let Some(event) = outcome {
            match event {
                Event::CompletionRequested => {
                    if interaction.editor().text() == "he" {
                        interaction.complete(0..2, "hello")?;
                    }
                    eprintln!("COMPLETION");
                }
                Event::Rejected(error) => {
                    interaction.external_output(Role::Warning, &error.to_string())?;
                    eprintln!("REJECTED {error}");
                }
                event => {
                    eprintln!("EVENT {event:?}");
                    if let Event::Submitted(ref text) = event {
                        println!("echo: {text}");
                        if !text.is_empty() {
                            interaction.editor_mut()?.admit_history(text)?;
                        }
                    }
                    eprintln!("CLOSED");
                    if app.is_none() {
                        if matches!(event, Event::EndOfInput) {
                            break;
                        }
                        interaction.editor_mut()?.clear();
                        interaction.open_driven(
                            &std::io::stdin(),
                            &std::io::stdout(),
                            Prompt::new("driven")?,
                        )?;
                    }
                }
            }
        }
        if app.is_some() {
            let hex: String = interaction
                .editor()
                .text()
                .as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            eprintln!(
                "STATE {} {} {} {}",
                interaction.is_open(),
                interaction.editor().cursor(),
                advances,
                hex
            );
        }
    }
    interaction.close()?;
    Ok(())
}
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn main() {
    eprintln!("This example requires a qualified Linux/macOS terminal backend.");
}
