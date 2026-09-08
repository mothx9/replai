use crate::{EditError, Editor};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Generic text emphasis. Roles carry no application meaning.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Role {
    /// Terminal-default foreground and background.
    #[default]
    Default,
    /// Strong emphasis.
    Strong,
    /// Primary accent.
    Accent,
    /// Secondary text.
    Dim,
    /// Positive notification.
    Success,
    /// Caution notification.
    Warning,
    /// Error notification.
    Error,
}

/// The initial text-only compatibility palette; never sets a background color.
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    color: bool,
}
impl Theme {
    /// Resolve styling from explicit terminal facts, without reading the environment.
    /// Presence of NO_COLOR (even empty), non-TTY output, or TERM=dumb disables color.
    pub fn new(output_is_tty: bool, no_color_present: bool, term: Option<&str>) -> Self {
        Self {
            color: output_is_tty && !no_color_present && term != Some("dumb"),
        }
    }
    /// Resolve styling from NO_COLOR and TERM for the supplied output capability.
    pub fn from_environment(output_is_tty: bool) -> Self {
        crate::capabilities::environment_theme(output_is_tty)
    }
    /// Return the SGR sequence for a generic role, or empty text when color is disabled.
    pub fn sequence(self, role: Role) -> &'static str {
        crate::protocol::style(self.color, role)
    }
}

/// Host-provided plain prompt content with generic composition and continuation.
#[derive(Clone, Debug)]
pub struct Prompt {
    label: String,
    state: String,
    continuation: String,
}
impl Prompt {
    /// Create `<accent>label><reset> ` with `... ` for logical continuations.
    /// Prompt fields reject control characters and are bounded to 1024 bytes each.
    pub fn new(label: &str) -> Result<Self, EditError> {
        prompt_text(label)?;
        Ok(Self {
            label: label.into(),
            state: String::new(),
            continuation: "... ".into(),
        })
    }
    /// Set a literal suffix between the label and `>`; spacing is host-provided.
    pub fn with_state(mut self, state: &str) -> Result<Self, EditError> {
        prompt_text(state)?;
        self.state = state.into();
        Ok(self)
    }
    /// Set the literal marker for each logical continuation line.
    pub fn with_continuation(mut self, marker: &str) -> Result<Self, EditError> {
        prompt_text(marker)?;
        self.continuation = marker.into();
        Ok(self)
    }
}
fn prompt_text(text: &str) -> Result<(), EditError> {
    if text.len() > 1024 {
        return Err(EditError::Capacity);
    }
    if text.chars().any(char::is_control) {
        return Err(EditError::InvalidText);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Point {
    pub row: usize,
    pub col: usize,
}
#[derive(Debug)]
pub(crate) struct Frame {
    pub lines: Vec<Line>,
    pub cursor: Point,
    pub end: Point,
}
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
            self.lines.last_mut().unwrap().text(g);
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
impl Frame {
    pub fn new(editor: &Editor, prompt: &Prompt, columns: usize, rows: usize) -> Self {
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
        Self {
            cursor: Point {
                row: cursor.row - start,
                col: cursor.col,
            },
            end: Point {
                row: end_row,
                col: l.widths[end - 1],
            },
            lines,
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Run {
    Text(String),
    Style(Role),
}

#[derive(Debug, Default, PartialEq)]
pub(crate) struct Line(pub(crate) Vec<Run>);
impl Line {
    fn text(&mut self, text: &str) {
        if let Some(Run::Text(previous)) = self.0.last_mut() {
            previous.push_str(text);
        } else {
            self.0.push(Run::Text(text.into()));
        }
    }
    pub fn suffix(&self, old: &Self) -> Option<&str> {
        if self == old {
            return Some("");
        }
        if self.0.len() != old.0.len() || self.0.is_empty() {
            return None;
        }
        let last = self.0.len() - 1;
        if self.0[..last] != old.0[..last] {
            return None;
        }
        if let (Run::Text(new), Run::Text(old)) = (&self.0[last], &old.0[last]) {
            return new.strip_prefix(old);
        }
        None
    }
}

#[cfg(test)]
#[path = "../tests/support/layout_reference.rs"]
mod reference;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layout_matches_frozen_full_document_oracle() {
        let units = [
            "a",
            "界",
            "e\u{301}",
            "👩\u{200d}💻",
            "🇮🇹",
            "\t",
            "\n",
            "\u{301}",
            "♥\u{fe0f}",
        ];
        let mut random = 19_u64;
        for case in 0..48 {
            let mut text = String::new();
            for _ in 0..case * 3 {
                random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
                text.push_str(units[(random >> 32) as usize % units.len()]);
            }
            let offsets: Vec<_> = text
                .grapheme_indices(true)
                .map(|(i, _)| i)
                .chain(std::iter::once(text.len()))
                .collect();
            let mut editor = Editor::new(4096, 0);
            editor.insert(&text).unwrap();
            let prompt = Prompt::new(if case % 2 == 0 { "p" } else { "long界prompt" })
                .unwrap()
                .with_state(" ready")
                .unwrap()
                .with_continuation("..界 ")
                .unwrap();
            for cursor in [offsets[0], offsets[offsets.len() / 2], text.len()] {
                editor.replace(cursor..cursor, "").unwrap();
                for columns in [2, 6, 20, 80] {
                    for rows in [2, 3, 24] {
                        let frame = Frame::new(&editor, &prompt, columns, rows);
                        let (lines, cursor, end) =
                            reference::frame(&editor, &prompt, columns, rows);
                        assert_eq!(frame.lines, lines, "case {case} at {columns}x{rows}");
                        assert_eq!(frame.cursor, cursor);
                        assert_eq!(frame.end, end);
                    }
                }
            }
        }
    }
    #[test]
    fn reference_palette_and_disable_rules_are_exact() {
        let roles = [
            Role::Default,
            Role::Strong,
            Role::Accent,
            Role::Dim,
            Role::Success,
            Role::Warning,
            Role::Error,
        ];
        let expected = [
            "\x1b[0m",
            "\x1b[1;38;5;250m",
            "\x1b[38;5;81m",
            "\x1b[38;5;245m",
            "\x1b[38;5;114m",
            "\x1b[38;5;179m",
            "\x1b[38;5;203m",
        ];
        for (role, sequence) in roles.into_iter().zip(expected) {
            assert_eq!(
                Theme::new(true, false, Some("xterm")).sequence(role),
                sequence
            );
            for theme in [
                Theme::new(true, true, None),
                Theme::new(true, false, Some("dumb")),
            ] {
                assert_eq!(theme.sequence(role), "");
            }
        }
    }
    #[test]
    fn cell_positions_ignore_styling_and_follow_graphemes() {
        let mut e = Editor::new(100, 1);
        e.insert("é界e\u{301}🌍").unwrap();
        e.left();
        let f = Frame::new(&e, &Prompt::new("demo").unwrap(), 80, 24);
        assert_eq!(f.cursor, Point { row: 0, col: 10 });
        assert_eq!(f.end, Point { row: 0, col: 12 });
        assert_eq!(
            crate::protocol::encode(&f.draw(), Theme::new(true, false, None)),
            "\x1b[38;5;81mdemo>\x1b[0m é界e\u{301}🌍\x1b[2D"
        );
    }
    #[test]
    fn wrapping_multiline_tabs_and_tall_drafts_have_bounded_geometry() {
        let mut e = Editor::new(100, 1);
        e.insert("界x\nq\t!").unwrap();
        let f = Frame::new(&e, &Prompt::new("d").unwrap(), 6, 24);
        assert_eq!(f.cursor, Point { row: 2, col: 3 });
        e.clear();
        e.insert("1\n2\n3\n4\n5").unwrap();
        let f = Frame::new(&e, &Prompt::new("d").unwrap(), 10, 3);
        assert_eq!(f.lines.len(), 2);
        assert!(f.cursor.row < 2);
        assert!(Prompt::new("bad\x1b[0m").is_err());
    }
    #[test]
    fn joined_emoji_layout_and_wrapped_prompt_style_remain_explicit() {
        let mut editor = Editor::new(100, 0);
        editor.insert("👩‍💻").unwrap();
        let theme = Theme::new(true, false, Some("xterm"));
        let frame = Frame::new(&editor, &Prompt::new("demo").unwrap(), 80, 24);
        assert_eq!(frame.cursor, Point { row: 0, col: 8 });
        editor.clear();
        let frame = Frame::new(&editor, &Prompt::new("abcdefghijklmnop").unwrap(), 6, 3);
        let mut parser = vt100::Parser::new(3, 6, 0);
        parser.process(crate::protocol::encode(&frame.draw(), theme).as_bytes());
        assert_eq!(
            parser.screen().cell(0, 0).unwrap().fgcolor(),
            vt100::Color::Idx(81)
        );
        assert_eq!(
            parser.screen().cell(1, 5).unwrap().fgcolor(),
            vt100::Color::Default
        );
    }
}
