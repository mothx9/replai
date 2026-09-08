//! Logical surface transitions; no terminal protocol or system resources.
use crate::{
    Editor, Prompt, Role,
    presentation::{Frame, Line, Point, Run},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

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
    ClearToEnd,
    ClearScreen,
    Paste(bool),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Damage {
    Rebuild,
    Cursor,
    AppendAscii,
    BackspaceAscii,
}

#[derive(Default)]
pub(crate) struct Renderer {
    frame: Option<Frame>,
}
impl Renderer {
    #[cfg(test)]
    pub(crate) fn assert_frame(&self, editor: &Editor, prompt: &Prompt, size: (usize, usize)) {
        let fresh = Frame::new(editor, prompt, size.0, size.1);
        let actual = self.frame.as_ref().unwrap();
        assert_eq!(
            actual.lines,
            fresh.lines,
            "draft {:?}, cursor {}",
            editor.text(),
            editor.cursor()
        );
        assert_eq!(actual.widths, fresh.widths);
        assert_eq!(
            actual.cursor,
            fresh.cursor,
            "draft {:?}, cursor {}",
            editor.text(),
            editor.cursor()
        );
        assert_eq!(actual.end, fresh.end);
        assert_eq!(
            (actual.source_len, actual.source_cursor),
            (fresh.source_len, fresh.source_cursor)
        );
    }
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
        self.transition(Frame::new(editor, prompt, size.0, size.1))
    }
    /// Reuse geometry only with a semantic invalidation supplied by Engine.
    /// Standalone characterization still constructs a fresh frame in `redraw`.
    pub fn redraw_changed(
        &mut self,
        editor: &Editor,
        prompt: &Prompt,
        size: (usize, usize),
        damage: Damage,
    ) -> Vec<Mutation> {
        if let Some(frame) = &mut self.frame
            && (frame.columns, frame.rows) == size
            && let Some(out) = frame.reuse(editor, damage)
        {
            return out;
        }
        self.redraw(editor, prompt, size)
    }
    // Separate already-built geometry from surface mutation for characterization.
    // The production redraw path calls this same transition seam.
    pub(crate) fn transition(&mut self, frame: Frame) -> Vec<Mutation> {
        if let Some(old) = &self.frame
            && old.columns == frame.columns
            && old.rows == frame.rows
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
        if let Some(old) = &self.frame
            && old.columns == frame.columns
            && old.rows == frame.rows
            && old.lines.len() == frame.lines.len()
        {
            let mut out = Vec::new();
            let mut at = old.cursor;
            for (row, (before, after)) in old.lines.iter().zip(&frame.lines).enumerate() {
                if before == after {
                    continue;
                }
                if let Some((col, suffix, role)) = ascii_suffix(before, after) {
                    goto(&mut out, &mut at, Point { row, col }, frame.columns);
                    out.push(Mutation::Style(role));
                    if !suffix.is_empty() {
                        out.push(Mutation::Text(suffix.into()));
                    }
                    if frame.widths[row] < old.widths[row] {
                        out.push(Mutation::ClearToEnd);
                    }
                } else {
                    // Absolute column anchoring also recovers from terminal
                    // emoji-width policies which differ from our logical model.
                    out.push(Mutation::CarriageReturn);
                    at.col = 0;
                    goto(&mut out, &mut at, Point { row, col: 0 }, frame.columns);
                    out.push(Mutation::ClearLine);
                    out.extend(after.0.iter().map(run_mutation));
                }
                at = Point {
                    row,
                    col: frame.widths[row],
                };
            }
            goto(&mut out, &mut at, frame.cursor, frame.columns);
            self.frame = Some(frame);
            return out;
        }
        let mut out = self.erase();
        out.extend(frame.draw());
        self.frame = Some(frame);
        out
    }
}

fn run_mutation(run: &Run) -> Mutation {
    match run {
        Run::Text(text) => Mutation::Text(text.clone()),
        Run::Style(role) => Mutation::Style(*role),
    }
}

// Prefix reuse is deliberately restricted to ASCII. Combining/ZWJ context and
// separate prompt segments must not be reinterpreted by a second width policy.
fn ascii_suffix<'a>(old: &Line, new: &'a Line) -> Option<(usize, &'a str, Role)> {
    if old.0.len() != new.0.len() || new.0.is_empty() {
        return None;
    }
    let last = new.0.len() - 1;
    if old.0[..last] != new.0[..last] {
        return None;
    }
    let (Run::Text(before), Run::Text(after)) = (&old.0[last], &new.0[last]) else {
        return None;
    };
    if !before.is_ascii() || !after.is_ascii() {
        return None;
    }
    let mut col = 0;
    let mut role = Role::Default;
    for run in &new.0[..last] {
        match run {
            Run::Text(t) if t.is_ascii() => col += t.len(),
            Run::Text(_) => return None,
            Run::Style(r) => role = *r,
        }
    }
    let shared = before
        .bytes()
        .zip(after.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    Some((col + shared, &after[shared..], role))
}

fn goto(out: &mut Vec<Mutation>, from: &mut Point, to: Point, columns: usize) {
    if *from == to {
        return;
    }
    if from.row == to.row && from.col < columns {
        if to.col < from.col {
            out.push(Mutation::Left(from.col - to.col));
        } else if to.col > from.col {
            out.push(Mutation::Right(to.col - from.col));
        }
    } else {
        // CR also cancels the terminal's pending wrap after a full-width row.
        out.push(Mutation::CarriageReturn);
        if from.row > to.row {
            out.push(Mutation::Up(from.row - to.row));
        } else if from.row < to.row {
            out.push(Mutation::Down(to.row - from.row));
        }
        if to.col > 0 {
            out.push(Mutation::Right(to.col));
        }
    }
    *from = to;
}

impl Frame {
    fn reuse(&mut self, editor: &Editor, damage: Damage) -> Option<Vec<Mutation>> {
        let text = editor.text();
        let mut out = Vec::new();
        match damage {
            Damage::Cursor if self.source_len == text.len() => {
                let old = self.source_cursor;
                let new = editor.cursor();
                if old == new {
                    return Some(out);
                }
                let span = &text[old.min(new)..old.max(new)];
                if span.len() > 4096 || span.contains(['\n', '\t']) {
                    return None;
                }
                let mut width = 0;
                for g in span.graphemes(true) {
                    let w = g.width();
                    if w > self.columns {
                        return None;
                    }
                    width += w;
                }
                let col = if new < old {
                    self.cursor.col.checked_sub(width)?
                } else {
                    self.cursor.col.checked_add(width)?
                };
                if col >= self.columns {
                    return None;
                }
                // At a pre-wrapped wide glyph, subtracting widths does not
                // describe the previous row. Keep those cases in full layout.
                if self.cursor.col == 0 {
                    return None;
                }
                if let Some(g) = text[new..].graphemes(true).next()
                    && g != "\n"
                    && g != "\t"
                    && col + g.width().min(self.columns) > self.columns
                {
                    return None;
                }
                let target = Point {
                    row: self.cursor.row,
                    col,
                };
                goto(&mut out, &mut self.cursor, target, self.columns);
            }
            Damage::AppendAscii
                if self.source_cursor == self.source_len
                    && self.cursor == self.end
                    && editor.cursor() == text.len() =>
            {
                let suffix = text.get(self.source_len..)?;
                if suffix.is_empty()
                    || !suffix.bytes().all(|b| b == b' ' || b.is_ascii_graphic())
                    || self.end.col + suffix.len() >= self.columns
                {
                    return None;
                }
                let Some(Run::Text(last)) = self.lines.last_mut()?.0.last_mut() else {
                    return None;
                };
                last.push_str(suffix);
                self.end.col += suffix.len();
                self.cursor = self.end;
                *self.widths.last_mut()? = self.end.col;
                out.push(Mutation::Text(suffix.into()));
            }
            Damage::BackspaceAscii
                if self.source_cursor == self.source_len
                    && self.cursor == self.end
                    && editor.cursor() == text.len() =>
            {
                let removed = self.source_len.checked_sub(text.len())?;
                if removed == 0 || removed > self.end.col {
                    return None;
                }
                let line = self.lines.last_mut()?;
                let Some(Run::Text(last)) = line.0.last_mut() else {
                    return None;
                };
                let keep = last.len().checked_sub(removed)?;
                if !last
                    .get(keep..)?
                    .bytes()
                    .all(|b| b == b' ' || b.is_ascii_graphic())
                {
                    return None;
                }
                last.truncate(keep);
                if last.is_empty() {
                    line.0.pop();
                }
                self.end.col -= removed;
                let target = self.end;
                goto(&mut out, &mut self.cursor, target, self.columns);
                *self.widths.last_mut()? = self.end.col;
                out.push(Mutation::ClearToEnd);
            }
            _ => return None,
        }
        self.source_len = text.len();
        self.source_cursor = editor.cursor();
        Some(out)
    }
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
            out.extend(line.0.iter().map(run_mutation));
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
