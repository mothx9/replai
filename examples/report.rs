//! Standalone build-report and command-help presentation using static example data.
//! This example runs no build, tests, packaging or deployment.
use replai::{Alignment, Block, Column, Document, ListItem, Role, Severity, Text, Theme};
use std::io::IsTerminal;

fn text(value: &str) -> Text {
    Text::new(value).expect("safe constant example text")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let document = Document::new(vec![
        Block::Heading {
            level: 1,
            text: text("Build console"),
        },
        Block::Paragraph(Text::styled(
            Role::Dim,
            "Static example data / no build or deployment is executed",
        )?),
        Block::Spacer,
        Block::KeyValue(vec![
            (text("Workspace"), text("terminal-tools")),
            (text("Profile"), text("release")),
            (text("Target"), text("aarch64-unknown-linux-gnu")),
        ]),
        Block::Spacer,
        Block::Heading {
            level: 2,
            text: text("Pipeline summary"),
        },
        Block::Table {
            columns: ["Stage", "Result", "Detail"]
                .into_iter()
                .map(|heading| Column {
                    heading: text(heading),
                    alignment: Alignment::Left,
                })
                .collect(),
            rows: vec![
                vec![
                    text("Compile"),
                    Text::styled(Role::Success, "passed")?,
                    text("Release artifacts produced"),
                ],
                vec![
                    text("Tests"),
                    Text::styled(Role::Success, "passed")?,
                    text("Unit and integration checks"),
                ],
                vec![
                    text("Package"),
                    Text::styled(Role::Warning, "review")?,
                    text("Signing key not configured"),
                ],
            ],
        },
        Block::Spacer,
        Block::Status {
            severity: Severity::Success,
            text: text("Local artifacts are ready for inspection."),
        },
        Block::Status {
            severity: Severity::Warning,
            text: text("Publication requires a signed package."),
        },
        Block::Spacer,
        Block::Heading {
            level: 2,
            text: text("Command help"),
        },
        Block::List {
            ordered: false,
            items: vec![
                ListItem {
                    depth: 0,
                    text: Text::from_spans(vec![
                        replai::Span::new(Role::Accent, "build ")?,
                        replai::Span::new(Role::Default, "Compile the selected workspace")?,
                    ])?,
                },
                ListItem {
                    depth: 0,
                    text: Text::from_spans(vec![
                        replai::Span::new(Role::Accent, "test ")?,
                        replai::Span::new(Role::Default, "Run checks before packaging")?,
                    ])?,
                },
                ListItem {
                    depth: 0,
                    text: Text::from_spans(vec![
                        replai::Span::new(Role::Accent, "inspect ")?,
                        replai::Span::new(Role::Default, "Show artifact details and diagnostics")?,
                    ])?,
                },
            ],
        },
    ])?;
    document.write_to(
        &mut std::io::stdout().lock(),
        100,
        Theme::from_environment(std::io::stdout().is_terminal()),
    )?;
    Ok(())
}
