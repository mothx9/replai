//! Whole-string reference model. No calls to private editor mutation/geometry helpers.
use crate::{EditError, Editor};
use std::{collections::VecDeque, ops::Range};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub const LIMIT: usize = 4096;
pub const WORDS: &[&str] = &[
    "",
    "a",
    "界",
    "e\u{301}",
    "\n",
    "\t",
    "\u{301}",
    "👩‍💻",
    "🇮",
    "🇹",
    "\u{600}",
    "क्ष",
    "\u{200d}",
    "x\ny",
    "\x1b",
    "\r",
    "\0",
];

#[derive(Default)]
pub struct Model {
    pub text: String,
    pub cursor: usize,
    history: VecDeque<String>,
    selected: Option<usize>,
    draft: Option<(String, usize)>,
}
impl Model {
    pub fn boundaries(&self) -> Vec<usize> {
        self.text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .chain([self.text.len()])
            .collect()
    }
    fn snap(&mut self, wanted: usize) {
        self.cursor = self
            .boundaries()
            .into_iter()
            .find(|i| *i >= wanted)
            .unwrap();
    }
    pub fn replace(&mut self, r: Range<usize>, text: &str) -> Result<(), EditError> {
        let b = self.boundaries();
        if r.start > r.end || !b.contains(&r.start) || !b.contains(&r.end) {
            return Err(EditError::InvalidRange);
        }
        if text
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
        {
            return Err(EditError::InvalidText);
        }
        if self.text.len() - r.len() + text.len() > LIMIT {
            return Err(EditError::Capacity);
        }
        let wanted = r.start + text.len();
        self.text.replace_range(r, text);
        self.snap(wanted);
        Ok(())
    }
    pub fn operation(&mut self, e: &mut Editor, op: u8, a: u8, b: u8, text: &str) {
        let before = e.analysis_snapshot();
        let boundaries = self.boundaries();
        match op % 16 {
            0 => assert_eq!(e.insert(text), self.replace(self.cursor..self.cursor, text)),
            1 | 2 => {
                let r = if op % 16 == 1 {
                    boundaries[a as usize % boundaries.len()]
                        ..boundaries[b as usize % boundaries.len()]
                } else {
                    a as usize..b as usize
                };
                assert_eq!(e.replace(r.clone(), text), self.replace(r, text));
            }
            3 => {
                e.left();
                self.cursor = boundaries
                    .into_iter()
                    .rfind(|i| *i < self.cursor)
                    .unwrap_or(0);
            }
            4 => {
                e.right();
                self.cursor = boundaries
                    .into_iter()
                    .find(|i| *i > self.cursor)
                    .unwrap_or(self.text.len());
            }
            5 => {
                e.home();
                self.cursor = 0;
            }
            6 => {
                e.end();
                self.cursor = self.text.len();
            }
            7 => {
                e.backspace();
                let start = boundaries
                    .into_iter()
                    .rfind(|i| *i < self.cursor)
                    .unwrap_or(0);
                self.replace(start..self.cursor, "").unwrap();
            }
            8 => {
                e.delete();
                let end = boundaries
                    .into_iter()
                    .find(|i| *i > self.cursor)
                    .unwrap_or(self.text.len());
                self.replace(self.cursor..end, "").unwrap();
            }
            9 => {
                e.clear();
                self.text.clear();
                self.cursor = 0;
                self.selected = None;
                self.draft = None;
            }
            10 => {
                let valid = !text
                    .chars()
                    .any(|c| c.is_control() && c != '\n' && c != '\t');
                let expected = if !valid {
                    Err(EditError::InvalidText)
                } else if text.len() > LIMIT {
                    Err(EditError::Capacity)
                } else {
                    Ok(())
                };
                assert_eq!(e.admit_history(text), expected);
                if expected.is_ok() {
                    if self.history.len() == 8 {
                        self.history.pop_front();
                    }
                    self.history.push_back(text.into());
                    self.selected = None;
                    self.draft = None;
                }
            }
            11 => {
                e.history_up();
                if !self.history.is_empty() && self.selected != Some(0) {
                    if self.selected.is_none() {
                        self.draft = Some((self.text.clone(), self.cursor));
                    }
                    let i = self.selected.map_or(self.history.len() - 1, |i| i - 1);
                    self.selected = Some(i);
                    self.text.clone_from(&self.history[i]);
                    self.cursor = self.text.len();
                }
            }
            12 => {
                e.history_down();
                if let Some(i) = self.selected {
                    if let Some(t) = self.history.get(i + 1) {
                        self.text.clone_from(t);
                        self.cursor = t.len();
                        self.selected = Some(i + 1);
                    } else {
                        (self.text, self.cursor) = self.draft.take().unwrap();
                        self.selected = None;
                    }
                }
            }
            13 | 14 => {
                let down = op % 16 == 14;
                let lines: Vec<&str> = self.text.split('\n').collect();
                let line = self.text[..self.cursor]
                    .bytes()
                    .filter(|b| *b == b'\n')
                    .count();
                let target = if down {
                    (line + 1 < lines.len()).then_some(line + 1)
                } else {
                    line.checked_sub(1)
                };
                assert_eq!(
                    if down { e.line_down() } else { e.line_up() },
                    target.is_some()
                );
                if let Some(target) = target {
                    let start: usize = lines[..line].iter().map(|s| s.len() + 1).sum();
                    let width = |g: &str, c: usize| {
                        if g == "\t" {
                            4 - c % 4
                        } else {
                            UnicodeWidthStr::width(g)
                        }
                    };
                    let desired = self.text[start..self.cursor]
                        .graphemes(true)
                        .fold(0, |c, g| c + width(g, c));
                    let mut col = 0;
                    self.cursor = lines[..target].iter().map(|s| s.len() + 1).sum();
                    for g in lines[target].graphemes(true) {
                        let n = width(g, col);
                        if col + n > desired {
                            break;
                        }
                        col += n;
                        self.cursor += g.len();
                    }
                }
            }
            _ => {
                let oversized = "x".repeat(LIMIT + 1);
                assert_eq!(e.insert(&oversized), Err(EditError::Capacity));
            }
        }
        assert_eq!((e.text(), e.cursor()), (self.text.as_str(), self.cursor));
        assert!(self.boundaries().contains(&e.cursor()));
        assert!(e.text().len() <= LIMIT);
        let changed = (before.text(), before.cursor()) != (e.text(), e.cursor());
        assert_eq!(before.revision() != e.revision(), changed, "op {op}");
    }
}
