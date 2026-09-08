//! Frozen pre-optimization full-document layout oracle from dbd9562.
//! Test-only: intentionally retains the original allocation and scanning strategy.
use super::{Editor, Line, Point, Prompt, Role, Run};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

struct Layout {
    lines: Vec<Line>,
    widths: Vec<usize>,
    point: Point,
    columns: usize,
    style: Option<Role>,
    wrapped: bool,
}
impl Layout {
    fn new(columns: usize) -> Self {
        Self {
            lines: vec![Line::default()],
            widths: vec![0],
            point: Point::default(),
            columns: columns.max(2),
            style: None,
            wrapped: false,
        }
    }
    fn newline(&mut self) {
        self.lines
            .push(Line(self.style.map(Run::Style).into_iter().collect()));
        self.widths.push(0);
        self.point.row += 1;
        self.point.col = 0;
        self.wrapped = false;
    }
    fn text(&mut self, text: &str) {
        for g in text.graphemes(true) {
            if g == "\t" {
                let spaces = 4 - self.point.col % 4;
                for _ in 0..spaces {
                    self.text(" ");
                }
                continue;
            }
            let width = g.width();
            let (g, width) = if width > self.columns {
                ("�", 1)
            } else {
                (g, width)
            };
            if self.point.col + width > self.columns {
                self.newline();
            }
            let line = self.lines.last_mut().unwrap();
            if let Some(Run::Text(previous)) = line.0.last_mut() {
                previous.push_str(g);
            } else {
                line.0.push(Run::Text(g.into()));
            }
            self.wrapped = false;
            self.point.col += width;
            *self.widths.last_mut().unwrap() = self.point.col;
            // Materialize a full-width line explicitly, avoiding pending autowrap.
            if self.point.col == self.columns {
                self.newline();
                self.wrapped = true;
            }
        }
    }
    fn style(&mut self, role: Role) {
        self.style = Some(role);
        self.lines.last_mut().unwrap().0.push(Run::Style(role));
    }
}
pub(super) fn frame(
    editor: &Editor,
    prompt: &Prompt,
    columns: usize,
    rows: usize,
) -> (Vec<Line>, Point, Point) {
    let mut l = Layout::new(columns);
    l.style(Role::Accent);
    l.text(&prompt.label);
    l.text(&prompt.state);
    l.text(">");
    l.style(Role::Default);
    l.text(" ");
    let mut cursor = l.point;
    for (offset, g) in editor.text().grapheme_indices(true) {
        if g != "\n" && g != "\t" && l.point.col + g.width().min(l.columns) > l.columns {
            l.newline();
        }
        if offset == editor.cursor() {
            cursor = l.point;
        }
        if g == "\n" {
            if !l.wrapped {
                l.newline();
            }
            l.wrapped = false;
            l.text(&prompt.continuation);
        } else {
            l.text(g);
        }
    }
    if editor.cursor() == editor.text().len() {
        cursor = l.point;
    }
    let visible = rows.saturating_sub(1).max(1);
    let start = cursor.row.saturating_add(1).saturating_sub(visible);
    let end = (start + visible).min(l.lines.len());
    let mut lines: Vec<_> = l.lines.drain(start..end).collect();
    // A viewport may begin inside a wrapped styled prompt.
    if start > 0 {
        lines[0].0.insert(0, Run::Style(Role::Default));
    }
    let end_row = lines.len() - 1;
    let cursor = Point {
        row: cursor.row - start,
        col: cursor.col,
    };
    let end = Point {
        row: end_row,
        col: l.widths[end - 1],
    };
    (lines, cursor, end)
}
