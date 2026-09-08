//! Differential semantic oracle for local-boundary/editor redesigns.
//! The oracle deliberately segments the complete text after each mutation.
use replai::{EditError, Editor};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

const LIMIT: usize = 2048;
const CORPUS: &[&str] = &[
    "",
    "a",
    "ab",
    "é",
    "界",
    "\n",
    "\t",
    "\u{301}",
    "e\u{301}",
    "\u{200d}",
    "\u{fe0f}",
    "👩",
    "💻",
    "👩\u{200d}💻",
    "🇮",
    "🇹",
    "🇫🇷🇮",
    "\u{600}",
    "क",
    "्",
    "क्ष",
    "ा",
    "♥\u{fe0f}",
    "1\u{fe0f}\u{20e3}",
];

#[derive(Default)]
struct Model {
    text: String,
    cursor: usize,
}
impl Model {
    fn boundaries(&self) -> Vec<usize> {
        self.text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .chain(std::iter::once(self.text.len()))
            .collect()
    }
    fn snap(&mut self, wanted: usize) {
        self.cursor = self
            .boundaries()
            .into_iter()
            .find(|i| *i >= wanted)
            .unwrap();
    }
    fn replace(&mut self, range: Range<usize>, replacement: &str) -> Result<(), EditError> {
        let boundaries = self.boundaries();
        if range.start > range.end
            || !boundaries.contains(&range.start)
            || !boundaries.contains(&range.end)
        {
            return Err(EditError::InvalidRange);
        }
        if replacement
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t')
        {
            return Err(EditError::InvalidText);
        }
        if self.text.len() - range.len() + replacement.len() > LIMIT {
            return Err(EditError::Capacity);
        }
        let wanted = range.start + replacement.len();
        self.text.replace_range(range, replacement);
        self.snap(wanted);
        Ok(())
    }
    fn check(&self, editor: &Editor) {
        assert_eq!(
            (editor.text(), editor.cursor()),
            (self.text.as_str(), self.cursor)
        );
        assert!(self.boundaries().contains(&editor.cursor()));
        assert!(editor.text().len() <= LIMIT);
    }
}

#[test]
fn insertion_replacement_and_deletion_resegment_both_neighbors() {
    for left in CORPUS {
        for right in CORPUS {
            for inserted in CORPUS {
                let original = format!("{left} {right}");
                let mut model = Model {
                    cursor: original.len(),
                    text: original.clone(),
                };
                let mut editor = Editor::new(LIMIT, 0);
                editor.insert(&original).unwrap();
                // Empty and nonempty replacements include cases that join RI,
                // Extend, Prepend, SpacingMark, variation selectors and ZWJ.
                let range = left.len()..left.len() + 1;
                assert_eq!(
                    editor.replace(range.clone(), inserted),
                    model.replace(range, inserted)
                );
                model.check(&editor);
                let old_cursor = model.cursor;
                let previous = model
                    .boundaries()
                    .into_iter()
                    .rfind(|i| *i < old_cursor)
                    .unwrap_or(0);
                editor.backspace();
                model.text.replace_range(previous..old_cursor, "");
                model.snap(previous);
                model.check(&editor);
            }
        }
    }
}

#[test]
fn arbitrary_bounded_edit_sequences_match_full_segmentation() {
    for seed in 0..16_u64 {
        let mut random = seed + 1;
        let mut next = || {
            random = random
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (random >> 32) as usize
        };
        let mut model = Model::default();
        let mut editor = Editor::new(LIMIT, 0);
        for step in 0..4096 {
            let text = CORPUS[next() % CORPUS.len()];
            let boundaries = model.boundaries();
            match next() % 10 {
                0..=2 => {
                    let range = model.cursor..model.cursor;
                    assert_eq!(
                        editor.insert(text),
                        model.replace(range, text),
                        "seed {seed} step {step}"
                    );
                }
                3 => {
                    let a = boundaries[next() % boundaries.len()];
                    let b = boundaries[next() % boundaries.len()];
                    let range = a.min(b)..a.max(b);
                    assert_eq!(
                        editor.replace(range.clone(), text),
                        model.replace(range, text)
                    );
                }
                4 => {
                    editor.left();
                    model.cursor = boundaries
                        .into_iter()
                        .rfind(|i| *i < model.cursor)
                        .unwrap_or(0);
                }
                5 => {
                    editor.right();
                    model.cursor = boundaries
                        .into_iter()
                        .find(|i| *i > model.cursor)
                        .unwrap_or(model.text.len());
                }
                6 => {
                    let end = model.cursor;
                    let start = boundaries.into_iter().rfind(|i| *i < end).unwrap_or(0);
                    editor.backspace();
                    model.text.replace_range(start..end, "");
                    model.snap(start);
                }
                7 => {
                    let start = model.cursor;
                    let end = boundaries
                        .into_iter()
                        .find(|i| *i > start)
                        .unwrap_or(model.text.len());
                    editor.delete();
                    model.text.replace_range(start..end, "");
                    model.snap(start);
                }
                8 => {
                    editor.home();
                    model.cursor = 0;
                }
                _ => {
                    editor.end();
                    model.cursor = model.text.len();
                }
            }
            model.check(&editor);
            // Malformed host offsets must fail atomically, including inside UTF-8.
            let offset = next() % (model.text.len() + 2);
            assert_eq!(
                editor.replace(offset..offset, ""),
                model.replace(offset..offset, "")
            );
            model.check(&editor);
        }
    }
}

#[test]
fn long_context_and_capacity_rejections_keep_exact_cursor_semantics() {
    for text in [
        "e".to_owned() + &"\u{301}".repeat(900),
        "🇮".repeat(500),
        "\u{600}".repeat(900) + "界",
    ] {
        let mut model = Model {
            cursor: text.len(),
            text: text.clone(),
        };
        let mut editor = Editor::new(LIMIT, 0);
        editor.insert(&text).unwrap();
        for replacement in ["X", "\u{200d}", "\u{fe0f}", "\u{301}", "🇹"] {
            for at_end in [false, true] {
                let cursor = if at_end { model.text.len() } else { 0 };
                editor.replace(cursor..cursor, "").unwrap();
                model.cursor = cursor;
                let range = cursor..cursor;
                assert_eq!(
                    editor.insert(replacement),
                    model.replace(range, replacement)
                );
                model.check(&editor);
            }
        }
        assert_eq!(editor.insert(&"x".repeat(LIMIT)), Err(EditError::Capacity));
        model.check(&editor);
        assert_eq!(editor.insert("\x1b"), Err(EditError::InvalidText));
        model.check(&editor);
    }
}
