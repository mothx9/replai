//! A bounded query-console host with fixture data; no database or SQL parser.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use replai::{
        Alignment, Block, Column, Document, Editor, Event, Interaction, Prompt, Role, Severity,
        Span, TerminalConfig, Text,
    };
    use std::time::{Duration, Instant};

    const QUERY: &str = "SELECT * FROM deployments;";
    let text = |s: &str| Text::new(s).expect("safe fixture text");
    let intro = Document::new(vec![
        Block::Heading {
            level: 1,
            text: text("Query console"),
        },
        Block::Paragraph(Text::styled(
            Role::Dim,
            "Local fixture / no database connection",
        )?),
        Block::KeyValue(vec![
            (text("Dataset"), text("deployments")),
            (
                text("Input"),
                text("Tab completes SELECT; /exit closes the console"),
            ),
        ]),
        Block::Spacer,
    ])?;
    let mut pending_output = Some(intro);
    let prompt = Prompt::composed(Text::from_spans(vec![
        Span::new(Role::Accent, "query")?,
        Span::new(Role::Dim, "[fixture]")?,
        Span::new(Role::Accent, "> ")?,
    ])?)?;
    let mut input = Interaction::new(Editor::new(65_536, 100));
    let started = Instant::now();
    let mut notice = std::env::args().any(|a| a == "--notice");
    loop {
        input.open_with_config(
            &std::io::stdin(),
            &std::io::stdout(),
            prompt.clone(),
            TerminalConfig::from_environment(),
        )?;
        if let Some(document) = pending_output.take() {
            input.output_document(&document)?;
        }
        loop {
            if notice && started.elapsed() >= Duration::from_secs(2) {
                input.external_output(
                    Role::Dim,
                    "Fixture notice: output arrived while this draft was open.",
                )?;
                notice = false;
            }
            match input.poll(Duration::from_millis(100))? {
                Some(Event::SubmissionRequested(_)) => unreachable!("direct submission"),
                None => {}
                Some(Event::CompletionRequested) => {
                    if QUERY.starts_with(input.editor().text()) {
                        input.complete(0..input.editor().text().len(), QUERY)?;
                    }
                }
                Some(Event::Submitted(line)) => {
                    let normalized = line.split_whitespace().collect::<Vec<_>>().join(" ");
                    if normalized == "/exit" {
                        return Ok(());
                    }
                    input.editor_mut()?.admit_history(&line)?;
                    input.editor_mut()?.clear();
                    let result = if normalized == QUERY {
                        Document::new(vec![
                            Block::Table {
                                columns: ["Service", "Region", "Replicas", "Status"]
                                    .into_iter()
                                    .map(|s| Column {
                                        heading: text(s),
                                        alignment: if s == "Replicas" {
                                            Alignment::Right
                                        } else {
                                            Alignment::Left
                                        },
                                    })
                                    .collect(),
                                rows: vec![
                                    vec![
                                        text("edge-api"),
                                        text("eu-west"),
                                        text("3"),
                                        Text::styled(Role::Success, "healthy")?,
                                    ],
                                    vec![
                                        text("worker"),
                                        text("us-east"),
                                        text("2"),
                                        Text::styled(Role::Warning, "degraded")?,
                                    ],
                                    vec![
                                        text("scheduler"),
                                        text("eu-west"),
                                        text("1"),
                                        Text::styled(Role::Success, "healthy")?,
                                    ],
                                ],
                            },
                            Block::Status {
                                severity: Severity::Success,
                                text: text("3 fixture records returned"),
                            },
                            Block::Spacer,
                        ])?
                    } else {
                        Document::new(vec![Block::Status {
                            severity: Severity::Info,
                            text: text("This fixture accepts SELECT * FROM deployments; or /exit."),
                        }])?
                    };
                    pending_output = Some(result);
                    break;
                }
                Some(Event::Interrupted) => {
                    input.editor_mut()?.clear();
                    break;
                }
                Some(Event::EndOfInput) => return Ok(()),
                Some(Event::Rejected(error)) => {
                    input.external_output(Role::Warning, &error.to_string())?
                }
            }
        }
    }
}
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn main() {
    eprintln!("This example requires a qualified Linux/macOS terminal backend.");
}
