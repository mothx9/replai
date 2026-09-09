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

/// Restrained foreground choices. Background always belongs to the terminal.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Foreground {
    /// Terminal foreground.
    #[default]
    Default,
    /// Neutral light foreground (256-color index 250).
    Neutral,
    /// Cyan accent (81).
    Cyan,
    /// Secondary gray (245).
    Gray,
    /// Green (114).
    Green,
    /// Amber (179).
    Amber,
    /// Red (203).
    Red,
}
/// Terminal appearance independently mapped from semantic roles. No raw escapes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Style {
    /// Foreground from the restrained terminal palette.
    pub foreground: Foreground,
    /// Strong intensity. No background or terminal-global mode changes.
    pub bold: bool,
}

/// The initial text-only compatibility palette; never sets a background color.
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub(crate) color: bool,
    styles: [Style; 7],
}
impl Theme {
    /// Resolve styling from explicit terminal facts, without reading the environment.
    /// Presence of NO_COLOR (even empty), non-TTY output, or TERM=dumb disables color.
    pub fn new(output_is_tty: bool, no_color_present: bool, term: Option<&str>) -> Self {
        Self {
            color: output_is_tty && !no_color_present && term != Some("dumb"),
            styles: [
                Style {
                    foreground: Foreground::Default,
                    bold: false,
                },
                Style {
                    foreground: Foreground::Neutral,
                    bold: true,
                },
                Style {
                    foreground: Foreground::Cyan,
                    bold: false,
                },
                Style {
                    foreground: Foreground::Gray,
                    bold: false,
                },
                Style {
                    foreground: Foreground::Green,
                    bold: false,
                },
                Style {
                    foreground: Foreground::Amber,
                    bold: false,
                },
                Style {
                    foreground: Foreground::Red,
                    bold: false,
                },
            ],
        }
    }
    /// Override one role's appearance. Color-disable policy still takes precedence.
    /// Default is the reset role and must retain terminal defaults.
    pub fn with_style(mut self, role: Role, style: Style) -> Result<Self, EditError> {
        if role == Role::Default && style != Style::default() {
            return Err(EditError::InvalidRange);
        }
        self.styles[role as usize] = style;
        Ok(self)
    }
    /// Inspect the resolved role appearance without terminal serialization.
    pub fn style(self, role: Role) -> Style {
        self.styles[role as usize]
    }
    /// Resolve styling from NO_COLOR and TERM for the supplied output capability.
    pub fn from_environment(output_is_tty: bool) -> Self {
        crate::capabilities::environment_theme(output_is_tty)
    }
    /// Return the SGR sequence for a generic role, or empty text when color is disabled.
    pub fn sequence(self, role: Role) -> &'static str {
        crate::protocol::style(self.color, self.styles[role as usize])
    }
}

/// Host-provided plain prompt content with generic composition and continuation.
#[derive(Clone, Debug)]
pub struct Prompt {
    label: String,
    state: String,
    continuation: String,
    composed: Option<crate::Text>,
    continued: Option<crate::Text>,
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
            composed: None,
            continued: None,
        })
    }
    /// Set a literal suffix between the label and `>`; spacing is host-provided.
    pub fn with_state(mut self, state: &str) -> Result<Self, EditError> {
        if self.composed.is_some() {
            return Err(EditError::InvalidRange);
        }
        prompt_text(state)?;
        self.state = state.into();
        Ok(self)
    }
    /// Set the literal marker for each logical continuation line.
    pub fn with_continuation(mut self, marker: &str) -> Result<Self, EditError> {
        prompt_text(marker)?;
        self.continuation = marker.into();
        self.continued = None;
        Ok(self)
    }
    /// Compose the complete primary prompt from safe styled segments, including
    /// host-chosen delimiters and trailing spacing. Maximum 3072 bytes, 64 spans;
    /// no controls. `with_state` applies only to the simple constructor.
    pub fn composed(text: crate::Text) -> Result<Self, EditError> {
        validate_segments(&text, 3072)?;
        let mut prompt = Self::new("")?;
        prompt.composed = Some(text);
        Ok(prompt)
    }
    /// Styled logical continuation marker, at most 1024 bytes and 64 spans.
    pub fn with_continuation_text(mut self, text: crate::Text) -> Result<Self, EditError> {
        validate_segments(&text, 1024)?;
        self.continued = Some(text);
        Ok(self)
    }
}
fn validate_segments(text: &crate::Text, limit: usize) -> Result<(), EditError> {
    if text.bytes() > limit || text.spans.len() > 64 {
        return Err(EditError::Capacity);
    }
    if text
        .spans
        .iter()
        .any(|s| s.text.chars().any(char::is_control))
    {
        return Err(EditError::InvalidText);
    }
    Ok(())
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
    pub widths: Vec<usize>,
    pub cursor: Point,
    pub end: Point,
    pub columns: usize,
    pub rows: usize,
    pub source_len: usize,
    pub source_cursor: usize,
}
// Keep only the potential viewport while locating the cursor. Recycle row
// storage while scanning its prefix; stop once the visible suffix is complete.
#[derive(Default)]
struct Row {
    line: Line,
    width: usize,
    spare: String,
}
impl Row {
    fn reset(&mut self, style: Option<Role>, reuse_limit: usize) {
        for run in self.line.0.drain(..) {
            if let Run::Text(text) = run
                && text.capacity() <= reuse_limit
                && text.capacity() > self.spare.capacity()
            {
                self.spare = text;
            }
        }
        self.spare.clear();
        self.width = 0;
        if let Some(role) = style {
            self.line.0.push(Run::Style(role));
        }
    }
    fn text(&mut self, text: &str) {
        if let Some(Run::Text(previous)) = self.line.0.last_mut() {
            previous.push_str(text);
        } else {
            let mut buffer = std::mem::take(&mut self.spare);
            buffer.push_str(text);
            self.line.0.push(Run::Text(buffer));
        }
    }
}
struct Layout {
    lines: std::collections::VecDeque<Row>,
    first_row: usize,
    visible: usize,
    stop_row: Option<usize>,
    full: bool,
    point: Point,
    columns: usize,
    style: Option<Role>,
    wrapped: bool,
}
impl Layout {
    fn new(columns: usize, visible: usize) -> Self {
        Self {
            lines: std::collections::VecDeque::from([Row::default()]),
            first_row: 0,
            visible,
            stop_row: None,
            full: false,
            point: Point::default(),
            columns: columns.max(2),
            style: None,
            wrapped: false,
        }
    }
    fn found_cursor(&mut self) {
        let start = self
            .point
            .row
            .saturating_add(1)
            .saturating_sub(self.visible);
        self.stop_row = Some(start.saturating_add(self.visible));
    }
    fn newline(&mut self) {
        self.point.row += 1;
        self.point.col = 0;
        self.wrapped = false;
        if self.stop_row.is_some_and(|end| self.point.row >= end) {
            self.full = true;
            return;
        }
        let mut row = if self.lines.len() == self.visible {
            self.first_row += 1;
            self.lines.pop_front().unwrap()
        } else {
            Row::default()
        };
        // Do not retain an offscreen giant combining grapheme in a tiny row.
        row.reset(self.style, self.columns.saturating_mul(4).max(4096));
        self.lines.push_back(row);
    }
    fn grapheme(&mut self, grapheme: &str, width: usize) {
        if self.full {
            return;
        }
        if grapheme == "\t" {
            let spaces = 4 - self.point.col % 4;
            for _ in 0..spaces {
                self.grapheme(" ", 1);
            }
            return;
        }
        let (text, width) = if width > self.columns {
            ("�", 1)
        } else {
            (grapheme, width)
        };
        if self.point.col + width > self.columns {
            self.newline();
            if self.full {
                return;
            }
        }
        let row = self.lines.back_mut().unwrap();
        row.text(text);
        self.wrapped = false;
        self.point.col += width;
        row.width = self.point.col;
        // Explicit full-width rows preserve the original autowrap policy.
        if self.point.col == self.columns {
            self.newline();
            self.wrapped = true;
        }
    }
    fn text(&mut self, text: &str) {
        for grapheme in text.graphemes(true) {
            if self.full {
                break;
            }
            self.grapheme(grapheme, grapheme.width());
        }
    }
    fn semantic(&mut self, text: &crate::Text) {
        let flat = text.plain();
        let mut span = 0;
        let mut end = text.spans.first().map_or(0, |s| s.text.len());
        let mut previous = None;
        for (offset, g) in flat.grapheme_indices(true) {
            while span + 1 < text.spans.len() && offset >= end {
                span += 1;
                end += text.spans[span].text.len();
            }
            let role = text.spans[span].role.unwrap_or(Role::Default);
            if previous != Some(role) {
                self.style(Role::Default);
                self.style(role);
                previous = Some(role);
            }
            self.grapheme(g, g.width());
        }
        self.style(Role::Default);
    }
    fn style(&mut self, role: Role) {
        self.style = Some(role);
        if !self.full {
            self.lines.back_mut().unwrap().line.0.push(Run::Style(role));
        }
    }
}
impl Frame {
    pub fn new(editor: &Editor, prompt: &Prompt, columns: usize, rows: usize) -> Self {
        let visible = rows.saturating_sub(1).max(1);
        let mut layout = Layout::new(columns, visible);
        if let Some(text) = &prompt.composed {
            layout.semantic(text);
        } else {
            layout.style(Role::Accent);
            layout.text(&prompt.label);
            layout.text(&prompt.state);
            layout.text(">");
            layout.style(Role::Default);
            layout.text(" ");
        }
        let mut cursor = None;
        for (offset, grapheme) in editor.text().grapheme_indices(true) {
            if layout.full {
                break;
            }
            let width = grapheme.width();
            // Preserve the pre-wrap cursor policy, including oversized glyphs.
            if grapheme != "\n"
                && grapheme != "\t"
                && layout.point.col + width.min(layout.columns) > layout.columns
            {
                layout.newline();
                if layout.full {
                    break;
                }
            }
            if offset == editor.cursor() {
                cursor = Some(layout.point);
                layout.found_cursor();
            }
            if grapheme == "\n" {
                if !layout.wrapped {
                    layout.newline();
                }
                layout.wrapped = false;
                if let Some(text) = &prompt.continued {
                    layout.semantic(text);
                } else {
                    layout.text(&prompt.continuation);
                }
            } else {
                layout.grapheme(grapheme, width);
            }
        }
        if editor.cursor() == editor.text().len() {
            cursor = Some(layout.point);
        }
        let cursor = cursor.expect("a valid editor cursor was laid out");
        let end_col = layout.lines.back().unwrap().width;
        let widths = layout.lines.iter().map(|row| row.width).collect();
        let mut lines: Vec<_> = layout.lines.into_iter().map(|row| row.line).collect();
        // A viewport may begin inside a wrapped styled prompt.
        if layout.first_row > 0 {
            lines[0].0.insert(0, Run::Style(Role::Default));
        }
        let end_row = lines.len() - 1;
        Self {
            cursor: Point {
                row: cursor.row - layout.first_row,
                col: cursor.col,
            },
            end: Point {
                row: end_row,
                col: end_col,
            },
            lines,
            widths,
            columns: columns.max(2),
            rows,
            source_len: editor.text().len(),
            source_cursor: editor.cursor(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Run {
    Text(String),
    Style(Role),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Line(pub(crate) Vec<Run>);
impl Line {
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
