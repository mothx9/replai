//! Serialized flow output: the terminal owns wrapping and scrollback reflow.
use replai::{Block, Document, Role, Text, Theme};
use std::io::{self, IsTerminal, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut stdout = io::stdout().lock();
    let theme = Theme::from_environment(stdout.is_terminal());
    let text = "café e\u{301} abcdefghijklmnopqrstuvwxyz0123456789 "
        .repeat(4)
        .trim_end()
        .to_owned();
    if arguments.first().is_some_and(|arg| arg == "--fragments") {
        for fragment in ["first ", "stream ", &text, "\nlogical break\n"] {
            stdout.write_all(
                Text::styled(Role::Accent, fragment)?
                    .render_flow(theme)?
                    .as_bytes(),
            )?;
        }
    } else {
        let document = Document::new(vec![
            Block::Paragraph(Text::new(&text)?),
            Block::Paragraph(Text::new("logical break")?),
        ])?;
        if arguments.first().is_some_and(|arg| arg == "--fixed") {
            document.write_to(&mut stdout, 20, theme)?;
        } else {
            document.write_flow_to(&mut stdout, theme)?;
        }
    }
    stdout.flush()?;
    Ok(())
}
