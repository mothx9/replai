//! Logical surface transitions; no terminal protocol or system resources.
use crate::{
    Editor, Prompt, Role,
    presentation::{Frame, Run},
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Mutation {
    Text(String),
    Style(Role),
    CarriageReturn,
    Newline,
    Up(usize),
    Down(usize),
    Right(usize),
    Left(usize),
    ClearLine,
    ClearScreen,
    Paste(bool),
}

#[derive(Default)]
pub(crate) struct Renderer {
    frame: Option<Frame>,
}
impl Renderer {
    pub fn erase(&mut self) -> Vec<Mutation> {
        self.frame.take().map_or_else(
            || vec![Mutation::CarriageReturn, Mutation::ClearLine],
            |f| f.erase(),
        )
    }
    pub fn clear(&mut self) -> Vec<Mutation> {
        self.frame = None;
        vec![Mutation::ClearScreen]
    }
    pub fn leave(&mut self, interrupted: bool) -> Vec<Mutation> {
        let mut out = Vec::new();
        if let Some(frame) = self.frame.take() {
            let down = frame.lines.len() - 1 - frame.cursor.row;
            if down > 0 {
                out.push(Mutation::Down(down));
            }
            out.push(Mutation::CarriageReturn);
            if frame.end.col > 0 {
                out.push(Mutation::Right(frame.end.col));
            }
        }
        if interrupted {
            out.push(Mutation::Text("^C".into()));
        }
        out.push(Mutation::Newline);
        out
    }
    pub fn redraw(
        &mut self,
        editor: &Editor,
        prompt: &Prompt,
        size: (usize, usize),
    ) -> Vec<Mutation> {
        let frame = Frame::new(editor, prompt, size.0, size.1);
        if let Some(old) = &self.frame
            && old.cursor == old.end
            && frame.cursor == frame.end
            && old.lines.len() == frame.lines.len()
            && old.lines[..old.lines.len() - 1] == frame.lines[..frame.lines.len() - 1]
            && let Some(suffix) = frame
                .lines
                .last()
                .unwrap()
                .suffix(old.lines.last().unwrap())
        {
            let out = if suffix.is_empty() {
                Vec::new()
            } else {
                vec![Mutation::Text(suffix.into())]
            };
            self.frame = Some(frame);
            return out;
        }
        let mut out = self.erase();
        out.extend(frame.draw());
        self.frame = Some(frame);
        out
    }
}

impl Frame {
    pub fn erase(&self) -> Vec<Mutation> {
        let mut out = vec![Mutation::CarriageReturn];
        if self.cursor.row > 0 {
            out.push(Mutation::Up(self.cursor.row));
        }
        for row in 0..self.lines.len() {
            out.push(Mutation::ClearLine);
            if row + 1 < self.lines.len() {
                out.extend([Mutation::Down(1), Mutation::CarriageReturn]);
            }
        }
        if self.lines.len() > 1 {
            out.push(Mutation::Up(self.lines.len() - 1));
        }
        out.push(Mutation::CarriageReturn);
        out
    }
    pub fn draw(&self) -> Vec<Mutation> {
        let mut out = Vec::new();
        for (i, line) in self.lines.iter().enumerate() {
            if i > 0 {
                out.push(Mutation::Newline);
            }
            out.extend(line.0.iter().map(|run| match run {
                Run::Text(text) => Mutation::Text(text.clone()),
                Run::Style(role) => Mutation::Style(*role),
            }));
        }
        if self.cursor == self.end {
            return out;
        }
        if self.cursor.row == self.end.row && self.cursor.col < self.end.col {
            out.push(Mutation::Left(self.end.col - self.cursor.col));
            return out;
        }
        out.push(Mutation::CarriageReturn);
        let up = self.lines.len() - 1 - self.cursor.row;
        if up > 0 {
            out.push(Mutation::Up(up));
        }
        if self.cursor.col > 0 {
            out.push(Mutation::Right(self.cursor.col));
        }
        out
    }
}
