//! Generic grouped help and setup facts from one semantic document.
use replai::{Alignment, Block, Column, Document, ListItem, Severity, Text, Theme};
use std::io::IsTerminal;
fn text(value: &str) -> Text {
    Text::new(value).expect("constant safe fixture")
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let width = std::env::args()
        .nth(1)
        .map(|s| s.parse())
        .transpose()?
        .unwrap_or(80);
    let document = Document::new(vec![
        Block::Heading {
            level: 1,
            text: text("APPLICATION"),
        },
        Block::Heading {
            level: 2,
            text: text("Connection"),
        },
        Block::KeyValue(vec![
            (text("Endpoint"), text("http://127.0.0.1:18001")),
            (text("Service"), text("demo-service")),
            (text("Locality"), text("loopback")),
        ]),
        Block::Spacer,
        Block::Heading {
            level: 2,
            text: text("Capabilities"),
        },
        Block::Status {
            severity: Severity::Success,
            text: text("Text"),
        },
        Block::Status {
            severity: Severity::Success,
            text: text("JSON"),
        },
        Block::Status {
            severity: Severity::Warning,
            text: text("Native functions unavailable"),
        },
        Block::Spacer,
        Block::Heading {
            level: 2,
            text: text("Commands"),
        },
        Block::List {
            ordered: false,
            items: vec![
                ListItem {
                    depth: 0,
                    text: text("General"),
                },
                ListItem {
                    depth: 1,
                    text: text("/help  Show help"),
                },
                ListItem {
                    depth: 1,
                    text: text("/exit  Exit"),
                },
                ListItem {
                    depth: 0,
                    text: text("Work"),
                },
                ListItem {
                    depth: 1,
                    text: text("/run  Run task"),
                },
                ListItem {
                    depth: 1,
                    text: text("/status  Inspect state"),
                },
            ],
        },
        Block::Spacer,
        Block::Table {
            columns: vec![
                Column {
                    heading: text("Operation"),
                    alignment: Alignment::Left,
                },
                Column {
                    heading: text("Available"),
                    alignment: Alignment::Left,
                },
            ],
            rows: vec![
                vec![text("Read"), text("yes")],
                vec![text("Write"), text("no: read-only endpoint")],
            ],
        },
        Block::Literal(text("{\"ready\": true}")),
    ])?;
    document.write_to(
        &mut std::io::stdout().lock(),
        width,
        Theme::from_environment(std::io::stdout().is_terminal()),
    )?;
    Ok(())
}
