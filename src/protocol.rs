//! VT mutation encoding. Reusable over any byte transport; no OS resources.
use crate::{Role, Theme, render::Mutation};
use std::fmt::Write;

pub(crate) fn style(color: bool, role: Role) -> &'static str {
    if !color {
        return "";
    }
    match role {
        Role::Default => "\x1b[0m",
        Role::Strong => "\x1b[1;38;5;250m",
        Role::Accent => "\x1b[38;5;81m",
        Role::Dim => "\x1b[38;5;245m",
        Role::Success => "\x1b[38;5;114m",
        Role::Warning => "\x1b[38;5;179m",
        Role::Error => "\x1b[38;5;203m",
    }
}
pub(crate) fn encode(mutations: &[Mutation], theme: Theme) -> String {
    let mut out = String::new();
    for mutation in mutations {
        match mutation {
            Mutation::Text(text) => out.push_str(text),
            Mutation::Style(role) => out.push_str(theme.sequence(*role)),
            Mutation::CarriageReturn => out.push('\r'),
            Mutation::Newline => out.push_str("\r\n"),
            Mutation::Up(n) => {
                let _ = write!(out, "\x1b[{n}A");
            }
            Mutation::Down(n) => {
                let _ = write!(out, "\x1b[{n}B");
            }
            Mutation::Right(n) => {
                let _ = write!(out, "\x1b[{n}C");
            }
            Mutation::Left(n) => {
                let _ = write!(out, "\x1b[{n}D");
            }
            Mutation::ClearLine => out.push_str("\x1b[2K"),
            Mutation::ClearToEnd => out.push_str("\x1b[K"),
            Mutation::ClearScreen => out.push_str("\x1b[2J\x1b[H"),
            Mutation::Paste(true) => out.push_str("\x1b[?2004h"),
            Mutation::Paste(false) => out.push_str("\x1b[?2004l"),
        }
    }
    out
}
