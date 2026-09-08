//! Bounded semantic documents; no terminal resources or application schemas.
use crate::{
    EditError, Error, Role, Theme,
    presentation::{Line, Run},
    render::Mutation,
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const FIELD: usize = 16 * 1024;
const BUDGET: usize = 1024 * 1024;
const ROWS: usize = 65_536;

/// A validated UTF-8 span carrying generic emphasis, never terminal commands.
#[derive(Clone, Debug)]
pub struct Span {
    pub(crate) role: Option<Role>,
    pub(crate) text: String,
}
impl Span {
    /// Normalize CRLF; admit LF and TAB, reject every other control. Maximum 16 KiB.
    pub fn new(role: Role, text: &str) -> Result<Self, EditError> {
        if text.len() > FIELD {
            return Err(EditError::Capacity);
        }
        let text = text.replace("\r\n", "\n");
        if !crate::core::valid_text(&text) {
            return Err(EditError::InvalidText);
        }
        Ok(Self {
            role: Some(role),
            text,
        })
    }
}
/// Inline semantic text. At most 1024 spans and 16 KiB combined UTF-8.
/// A grapheme crossing a span boundary takes the role of its first scalar.
#[derive(Clone, Debug)]
pub struct Text {
    pub(crate) spans: Vec<Span>,
}
impl Text {
    /// Text using its block's default emphasis (terminal-default in paragraphs
    /// and composed prompts). Use `styled` to override, including Role::Default.
    pub fn new(text: &str) -> Result<Self, EditError> {
        let mut span = Span::new(Role::Default, text)?;
        span.role = None;
        Self::from_spans(vec![span])
    }
    /// One styled span.
    pub fn styled(role: Role, text: &str) -> Result<Self, EditError> {
        Self::from_spans(vec![Span::new(role, text)?])
    }
    /// Compose validated inline spans, preserving their text and order.
    pub fn from_spans(spans: Vec<Span>) -> Result<Self, EditError> {
        if spans.len() > 1024 || spans.iter().map(|s| s.text.len()).sum::<usize>() > FIELD {
            return Err(EditError::Capacity);
        }
        Ok(Self { spans })
    }
    pub(crate) fn plain(&self) -> String {
        self.spans.iter().map(|s| s.text.as_str()).collect()
    }
    pub(crate) fn bytes(&self) -> usize {
        self.spans.iter().map(|s| s.text.len()).sum()
    }
}
/// Cell alignment within a table column.
#[derive(Clone, Copy, Debug, Default)]
pub enum Alignment {
    /// Start alignment.
    #[default]
    Left,
    /// End alignment, useful for quantities.
    Right,
}
/// Table metadata supplied by the host.
#[derive(Clone, Debug)]
pub struct Column {
    /// Safe column heading.
    pub heading: Text,
    /// Alignment of values (headings remain start-aligned).
    pub alignment: Alignment,
}
/// One list entry with explicit, nonrecursive indentation.
#[derive(Clone, Debug)]
pub struct ListItem {
    /// Zero-based indentation depth, at most eight.
    pub depth: u8,
    /// Safe entry text.
    pub text: Text,
}
/// Generic notice classification. Each has a textual marker even without color.
#[derive(Clone, Copy, Debug)]
pub enum Severity {
    /// Informational notice (`[i]`).
    Info,
    /// Positive notice (`[ok]`).
    Success,
    /// Caution (`[!]`).
    Warning,
    /// Failure (`[error]`).
    Error,
}
/// A line-oriented semantic block, never a product-specific record or recursive DOM.
#[derive(Clone, Debug)]
pub enum Block {
    /// Wrapped text; embedded LF is preserved. Wrapping is at grapheme boundaries.
    Paragraph(Text),
    /// Levels 1–3, represented by one to three `#` markers and strong default text.
    Heading {
        /// Hierarchy level.
        level: u8,
        /// Safe heading text.
        text: Text,
    },
    /// An aligned group of label/value facts, stacked on narrow surfaces.
    KeyValue(Vec<(Text, Text)>),
    /// Ordered or unordered entries; depth is explicit and bounded.
    List {
        /// Use numeric markers instead of `-`.
        ordered: bool,
        /// Entries in presentation order.
        items: Vec<ListItem>,
    },
    /// Responsive table; cells may contain LF/TAB and are never silently truncated.
    Table {
        /// Between one and 32 columns.
        columns: Vec<Column>,
        /// At most 4096 rows, each matching the column count.
        rows: Vec<Vec<Text>>,
    },
    /// Safe literal data, preserving spaces and line breaks; wraps with a `| ` gutter.
    Literal(Text),
    /// Notice with a severity-specific textual marker and role.
    Status {
        /// Generic classification, chosen by the host.
        severity: Severity,
        /// Notice content.
        text: Text,
    },
    /// One explicit blank line.
    Spacer,
}
/// Immutable validated document, at most 4096 blocks and 1 MiB of text.
/// Rendering additionally bounds width (2–4096), rows (65,536), and encoded
/// bytes (8 MiB). Excess work returns Capacity before any writer is touched.
#[derive(Clone, Debug)]
pub struct Document {
    blocks: Vec<Block>,
}
impl Document {
    /// Validate structure and aggregate resource bounds without recursive traversal.
    pub fn new(blocks: Vec<Block>) -> Result<Self, EditError> {
        if blocks.len() > 4096 {
            return Err(EditError::Capacity);
        }
        let mut bytes = 0;
        let mut spans = 0;
        for b in &blocks {
            let texts: Vec<&Text> = match b {
                Block::Paragraph(t) | Block::Literal(t) | Block::Status { text: t, .. } => vec![t],
                Block::Heading { level, text } => {
                    if !(1..=3).contains(level) {
                        return Err(EditError::InvalidRange);
                    }
                    vec![text]
                }
                Block::KeyValue(fields) => {
                    if fields.len() > 4096 {
                        return Err(EditError::Capacity);
                    }
                    fields.iter().flat_map(|(k, v)| [k, v]).collect()
                }
                Block::List { items, .. } => {
                    if items.len() > 4096 {
                        return Err(EditError::Capacity);
                    }
                    if items.iter().any(|i| i.depth > 8) {
                        return Err(EditError::InvalidRange);
                    }
                    items.iter().map(|i| &i.text).collect()
                }
                Block::Table { columns, rows } => {
                    if columns.is_empty() {
                        return Err(EditError::InvalidRange);
                    }
                    if columns.len() > 32 || rows.len() > 4096 {
                        return Err(EditError::Capacity);
                    }
                    if rows.iter().any(|r| r.len() != columns.len()) {
                        return Err(EditError::InvalidRange);
                    }
                    columns
                        .iter()
                        .map(|c| &c.heading)
                        .chain(rows.iter().flatten())
                        .collect()
                }
                Block::Spacer => vec![],
            };
            for t in texts {
                bytes += t.bytes();
                spans += t.spans.len().max(1);
            }
            if bytes > BUDGET || spans > 16_384 {
                return Err(EditError::Capacity);
            }
        }
        Ok(Self { blocks })
    }
    /// Render without acquiring a terminal. Explicit width makes captured output
    /// deterministic; Theme resolves styling independently of document structure.
    /// Output uses LF line endings, including a final LF for every rendered row.
    pub fn render(&self, columns: usize, theme: Theme) -> Result<String, EditError> {
        let lines = self.layout(columns)?;
        let mut out = String::new();
        for line in &lines {
            for run in &line.0 {
                match run {
                    Run::Text(t) => out.push_str(t),
                    Run::Style(r) => out.push_str(theme.sequence(*r)),
                }
            }
            out.push('\n');
        }
        if out.len() > 8 * BUDGET {
            return Err(EditError::Capacity);
        }
        Ok(out)
    }
    /// Fully validate/layout before writing one batch. Writer failures propagate;
    /// the caller owns the writer and any policy for partially delivered output.
    pub fn write_to(
        &self,
        writer: &mut impl std::io::Write,
        columns: usize,
        theme: Theme,
    ) -> Result<(), Error> {
        let text = self.render(columns, theme)?;
        writer.write_all(text.as_bytes())?;
        Ok(())
    }
    pub(crate) fn mutations(&self, columns: usize) -> Result<Vec<Mutation>, EditError> {
        Ok(line_mutations(self.layout(columns)?))
    }
    pub(crate) fn layout(&self, columns: usize) -> Result<Vec<Line>, EditError> {
        if !(2..=4096).contains(&columns) {
            return Err(EditError::InvalidRange);
        }
        let mut out = Output::default();
        for block in &self.blocks {
            match block {
                Block::Paragraph(t) => append(&mut out, wrap(t, columns, Role::Default)?)?,
                Block::Heading { level, text } => prefixed(
                    &mut out,
                    text,
                    &format!("{} ", "#".repeat(*level as usize)),
                    columns,
                    Role::Strong,
                )?,
                Block::Literal(t) => prefixed(&mut out, t, "| ", columns, Role::Dim)?,
                Block::Spacer => append(&mut out, vec![Line::default()])?,
                Block::Status { severity, text } => {
                    let (prefix, role) = match severity {
                        Severity::Info => ("[i] ", Role::Accent),
                        Severity::Success => ("[ok] ", Role::Success),
                        Severity::Warning => ("[!] ", Role::Warning),
                        Severity::Error => ("[error] ", Role::Error),
                    };
                    prefixed(&mut out, text, prefix, columns, role)?;
                }
                Block::List { ordered, items } => {
                    let mut counts = [0usize; 9];
                    for item in items {
                        let depth = item.depth as usize;
                        counts[depth] += 1;
                        counts[depth + 1..].fill(0);
                        let marker = if *ordered {
                            format!("{}. ", counts[depth])
                        } else {
                            "- ".into()
                        };
                        prefixed(
                            &mut out,
                            &item.text,
                            &format!("{}{}", " ".repeat(depth * 2), marker),
                            columns,
                            Role::Default,
                        )?;
                    }
                }
                Block::KeyValue(fields) => {
                    facts(&mut out, fields.iter().map(|(k, v)| (k, v)), columns)?
                }
                Block::Table {
                    columns: cols,
                    rows,
                } => table(&mut out, cols, rows, columns)?,
            }
        }
        Ok(out.lines)
    }
}
#[derive(Default)]
struct Output {
    lines: Vec<Line>,
    work: usize,
}
fn append(out: &mut Output, rows: Vec<Line>) -> Result<(), EditError> {
    let work: usize = rows
        .iter()
        .map(|l| {
            l.0.iter()
                .map(|r| match r {
                    Run::Text(t) => t.len(),
                    Run::Style(_) => 32,
                })
                .sum::<usize>()
                + 2
        })
        .sum();
    if out.lines.len() + rows.len() > ROWS || out.work + work > 8 * BUDGET {
        return Err(EditError::Capacity);
    }
    out.work += work;
    out.lines.extend(rows);
    Ok(())
}

fn put(line: &mut Line, role: Role, text: &str) {
    // Always reset before changing emphasis; bold must not leak between spans.
    if !matches!(line.0.last(), Some(Run::Text(_)))
        || !matches!(line.0.get(line.0.len().saturating_sub(2)),Some(Run::Style(r)) if *r==role)
    {
        line.0.push(Run::Style(Role::Default));
        line.0.push(Run::Style(role));
        line.0.push(Run::Text(String::new()));
    }
    if let Some(Run::Text(t)) = line.0.last_mut() {
        t.push_str(text);
    }
}
fn width(line: &Line) -> usize {
    line.0
        .iter()
        .map(|r| match r {
            Run::Text(t) => t.width(),
            _ => 0,
        })
        .sum()
}
fn wrap(text: &Text, columns: usize, default: Role) -> Result<Vec<Line>, EditError> {
    let flat = text.plain();
    let mut rows = vec![Line::default()];
    let mut col = 0;
    let mut span = 0;
    let mut end = text.spans.first().map_or(0, |s| s.text.len());
    for (offset, g) in flat.grapheme_indices(true) {
        while span + 1 < text.spans.len() && offset >= end {
            span += 1;
            end += text.spans[span].text.len();
        }
        let role = text.spans.get(span).and_then(|s| s.role).unwrap_or(default);
        if g == "\n" {
            rows.push(Line::default());
            col = 0;
            continue;
        }
        let spaces = " ".repeat(if g == "\t" { 4 - col % 4 } else { 0 });
        let value = if g == "\t" { spaces.as_str() } else { g };
        // Tabs expand as spaces; other extended graphemes stay indivisible.
        for part in value.graphemes(true) {
            let n = part.width();
            if n > columns {
                return Err(EditError::InvalidRange);
            }
            if col + n > columns {
                rows.push(Line::default());
                col = 0;
            }
            put(rows.last_mut().unwrap(), role, part);
            col += n;
        }
        if rows.len() > ROWS {
            return Err(EditError::Capacity);
        }
    }
    for row in &mut rows {
        row.0.push(Run::Style(Role::Default));
    }
    Ok(rows)
}
fn prefixed(
    out: &mut Output,
    text: &Text,
    prefix: &str,
    columns: usize,
    role: Role,
) -> Result<(), EditError> {
    let indent = prefix.width();
    if indent + 2 > columns {
        append(
            out,
            wrap(&Text::styled(role, prefix.trim_end())?, columns, role)?,
        )?;
        return append(out, wrap(text, columns, Role::Default)?);
    }
    let rows = wrap(text, columns - indent, role)?;
    let mut result = Vec::new();
    for (i, row) in rows.into_iter().enumerate() {
        let mut line = Line::default();
        put(&mut line, role, if i == 0 { prefix } else { "" });
        if i > 0 {
            put(&mut line, Role::Default, &" ".repeat(indent));
        }
        line.0.extend(row.0);
        result.push(line);
    }
    append(out, result)
}
fn natural(text: &Text) -> usize {
    text.plain()
        .split('\n')
        .map(|s| s.replace('\t', "    ").width())
        .max()
        .unwrap_or(0)
}
fn facts<'a>(
    out: &mut Output,
    fields: impl Iterator<Item = (&'a Text, &'a Text)> + Clone,
    columns: usize,
) -> Result<(), EditError> {
    let label = fields.clone().map(|(k, _)| natural(k)).max().unwrap_or(0);
    if label + 2 + 4 > columns || label > columns / 2 {
        for (k, v) in fields {
            append(out, wrap(k, columns, Role::Strong)?)?;
            prefixed(out, v, "  ", columns, Role::Default)?;
        }
        return Ok(());
    }
    for (k, v) in fields {
        let left = wrap(k, label.max(2), Role::Strong)?;
        let right = wrap(v, columns - label - 2, Role::Default)?;
        for i in 0..left.len().max(right.len()) {
            let mut line = Line::default();
            if let Some(l) = left.get(i) {
                line.0.extend(l.0.clone());
            }
            let used = width(&line);
            put(&mut line, Role::Default, &" ".repeat(label + 2 - used));
            if let Some(r) = right.get(i) {
                line.0.extend(r.0.clone());
            }
            append(out, vec![line])?;
        }
    }
    Ok(())
}
fn table(
    out: &mut Output,
    cols: &[Column],
    rows: &[Vec<Text>],
    columns: usize,
) -> Result<(), EditError> {
    let n = cols.len();
    let gap = (n - 1) * 2;
    if n * 4 + gap > columns {
        if rows.is_empty() {
            for col in cols {
                append(out, wrap(&col.heading, columns, Role::Strong)?)?;
            }
        }
        for (i, row) in rows.iter().enumerate() {
            if i > 0 {
                append(out, vec![Line::default()])?;
            }
            facts(
                out,
                cols.iter().zip(row).map(|(c, t)| (&c.heading, t)),
                columns,
            )?;
        }
        return Ok(());
    }
    let mut widths: Vec<usize> = cols
        .iter()
        .map(|c| natural(&c.heading).max(4).min(columns))
        .collect();
    for row in rows {
        for (i, t) in row.iter().enumerate() {
            widths[i] = widths[i].max(natural(t).min(columns));
        }
    }
    // At most 32 * 4096 bounded iterations, independent of row count.
    while widths.iter().sum::<usize>() + gap > columns {
        let i = widths.iter().enumerate().max_by_key(|(_, w)| *w).unwrap().0;
        widths[i] -= 1;
    }
    let headings: Vec<Text> = cols.iter().map(|c| c.heading.clone()).collect();
    for (index, row) in std::iter::once(&headings).chain(rows).enumerate() {
        let cells: Vec<Vec<Line>> = row
            .iter()
            .enumerate()
            .map(|(i, t)| {
                wrap(
                    t,
                    widths[i],
                    if index == 0 {
                        Role::Strong
                    } else {
                        Role::Default
                    },
                )
            })
            .collect::<Result<_, _>>()?;
        for r in 0..cells.iter().map(Vec::len).max().unwrap_or(0) {
            let mut line = Line::default();
            for (i, cell) in cells.iter().enumerate() {
                let value = cell.get(r);
                let used = value.map_or(0, width);
                let padding = widths[i] - used;
                let right = index > 0 && matches!(cols[i].alignment, Alignment::Right);
                if right {
                    put(&mut line, Role::Default, &" ".repeat(padding));
                }
                if let Some(v) = value {
                    line.0.extend(v.0.clone());
                }
                if !right && i + 1 < n {
                    put(&mut line, Role::Default, &" ".repeat(padding));
                }
                if i + 1 < n {
                    put(&mut line, Role::Default, "  ");
                }
            }
            append(out, vec![line])?;
        }
        if index == 0 {
            let mut line = Line::default();
            put(
                &mut line,
                Role::Dim,
                &widths
                    .iter()
                    .map(|n| "-".repeat(*n))
                    .collect::<Vec<_>>()
                    .join("  "),
            );
            append(out, vec![line])?;
        }
    }
    Ok(())
}
pub(crate) fn line_mutations(lines: Vec<Line>) -> Vec<Mutation> {
    let mut out = Vec::new();
    for line in lines {
        for run in line.0 {
            out.push(match run {
                Run::Text(t) => Mutation::Text(t),
                Run::Style(r) => Mutation::Style(r),
            });
        }
        out.push(Mutation::Newline);
    }
    out
}

// Legacy unwrapped literal projection, preserving existing trailing-LF/TAB bytes.
pub(crate) fn plain_mutations(role: Role, text: &str) -> Result<Vec<Mutation>, EditError> {
    let text = text.replace("\r\n", "\n");
    if !crate::core::valid_text(&text) {
        return Err(EditError::InvalidText);
    }
    let mut mutations = vec![Mutation::Style(role)];
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            mutations.push(Mutation::Newline);
        }
        if !line.is_empty() {
            mutations.push(Mutation::Text(line.into()));
        }
    }
    mutations.push(Mutation::Style(Role::Default));
    if !text.ends_with('\n') {
        mutations.push(Mutation::Newline);
    }
    Ok(mutations)
}
