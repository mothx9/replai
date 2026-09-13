//! Public daily-driver editing and provider boundary regressions.
use replai::{
    AnalysisOutcome, EditError, Editor, EditorLimits, EditorLimitsError, HistoryError,
    HistoryProvider, HistoryProviderError, HistorySearchLimits, HistorySearchSource,
};
use unicode_segmentation::UnicodeSegmentation;

fn grapheme_boundary(text: &str, cursor: usize) -> bool {
    text.grapheme_indices(true)
        .map(|(i, _)| i)
        .chain([text.len()])
        .any(|i| i == cursor)
}

#[test]
fn unicode_word_operations_are_grapheme_safe_and_revisioned() {
    for text in [
        "alpha,  beta",
        "café e\u{301}lan",
        "東京 テスト",
        "one\ntwo",
        "x\t界",
        "emoji 👩\u{200d}💻 word",
        "flag 🇮🇹 end",
        "Latin Ελληνικά हिंदी",
    ] {
        let mut editor = Editor::new(1024, 0);
        editor.insert(text).unwrap();
        let end = editor.cursor();
        editor.word_left();
        assert!(grapheme_boundary(editor.text(), editor.cursor()));
        assert!(editor.cursor() <= end);
        let revision = editor.revision();
        editor.word_right();
        assert!(grapheme_boundary(editor.text(), editor.cursor()));
        if editor.cursor() != end {
            assert_ne!(editor.revision(), revision);
        }
        editor.delete_word_backward();
        assert!(grapheme_boundary(editor.text(), editor.cursor()));
        editor.undo();
        assert_eq!(editor.text(), text);
    }
}

#[test]
fn grouped_delta_undo_redo_never_reuses_revision() {
    let mut editor = Editor::new(1 << 20, 4);
    let initial = editor.analysis_snapshot();
    for part in ["h", "e", "l", "l", "o"] {
        editor.insert(part).unwrap();
    }
    assert_eq!(editor.undo_usage().0, 1);
    let typed = editor.analysis_snapshot();
    assert!(editor.undo());
    assert_eq!(editor.text(), "");
    let undone = editor.analysis_snapshot();
    assert_ne!(initial.revision(), undone.revision());
    assert_eq!(
        editor.replace_at(initial.revision(), 0..0, "bad"),
        Ok(AnalysisOutcome::Stale)
    );
    assert!(editor.redo());
    assert_eq!(editor.text(), "hello");
    assert_ne!(typed.revision(), editor.revision());
    assert!(editor.undo());
    editor.insert("different").unwrap();
    assert!(!editor.redo());
}

#[test]
fn replacements_deletions_and_cursor_only_changes_have_explicit_undo_policy() {
    let mut editor = Editor::new(128, 0);
    editor.insert("abc def").unwrap();
    editor.undo();
    editor.redo();
    editor.home();
    let before = editor.revision();
    editor.word_right();
    assert_ne!(before, editor.revision());
    editor.delete_word_forward();
    assert_eq!(editor.text(), "abc");
    assert!(editor.undo());
    assert_eq!(editor.text(), "abc def");
    assert_eq!(editor.cursor(), 3);
    editor.right();
    assert!(editor.redo());
    assert_eq!(editor.text(), "abc");
}

#[test]
fn kill_yank_is_bounded_atomic_and_undoable() {
    let limits = EditorLimits::new(16, 8, 32, 4).unwrap();
    let mut editor = Editor::with_limits(16, 0, limits).unwrap();
    editor.insert("one two").unwrap();
    editor.kill_word_backward().unwrap();
    assert_eq!((editor.text(), editor.kill_register()), ("one ", "two"));
    assert!(editor.undo());
    assert_eq!(editor.text(), "one two");
    assert!(editor.redo());
    editor.yank().unwrap();
    assert_eq!(editor.text(), "one two");
    assert!(editor.undo());
    assert_eq!(editor.text(), "one ");

    editor.clear();
    editor.insert("12345").unwrap();
    let before = editor.analysis_snapshot();
    let register = editor.kill_register().to_owned();
    assert_eq!(editor.kill_word_backward(), Err(EditError::Capacity));
    assert_eq!(
        (editor.text(), editor.cursor(), editor.revision()),
        (before.text(), before.cursor(), before.revision())
    );
    assert_eq!(editor.kill_register(), register);
}

struct Fixture {
    entries: Vec<String>,
    fail_at: Option<usize>,
}
impl HistoryProvider for Fixture {
    fn entry(&mut self, index: usize) -> Result<Option<String>, HistoryProviderError> {
        if self.fail_at == Some(index) {
            return Err(HistoryProviderError);
        }
        Ok(self.entries.get(index).cloned())
    }
}

#[test]
fn provider_view_is_newest_first_bounded_validated_and_atomic() {
    let limits = HistorySearchLimits::new(2, 12, 8).unwrap();
    let mut fixture = Fixture {
        entries: vec!["new".into(), "older".into(), "ignored".into()],
        fail_at: None,
    };
    let source = HistorySearchSource::from_provider(&mut fixture, limits).unwrap();
    assert_eq!(
        (source.len(), source.retained_bytes(), source.limits()),
        (2, 8, limits)
    );

    let mut invalid = Fixture {
        entries: vec!["ok".into(), "bad\u{1b}".into()],
        fail_at: None,
    };
    assert!(matches!(
        HistorySearchSource::from_provider(&mut invalid, limits),
        Err(HistoryError::InvalidText)
    ));
    let mut failed = Fixture {
        entries: vec!["ok".into()],
        fail_at: Some(0),
    };
    assert!(matches!(
        HistorySearchSource::from_provider(&mut failed, limits),
        Err(HistoryError::Provider)
    ));
}

#[test]
fn editor_limits_are_inspectable_and_legacy_constructor_stays_coherent() {
    assert_eq!(EditorLimits::new(16, 0, 32, 4), Err(EditorLimitsError));
    assert_eq!(EditorLimits::new(16, 8, 31, 4), Err(EditorLimitsError));
    let editor = Editor::new(16, 2);
    assert_eq!(editor.capacity(), 16);
    assert_eq!(editor.limits().undo_entries(), 256);
    assert_eq!(editor.limits().undo_bytes(), 32);
    assert_eq!(editor.limits().kill_bytes(), 16);
}

#[test]
fn pre_ergonomics_edit_error_matches_remain_exhaustive() {
    fn classify(error: EditError) -> u8 {
        match error {
            EditError::Capacity => 0,
            EditError::InvalidText => 1,
            EditError::InvalidRange => 2,
            EditError::HistoryDisabled => 3,
            EditError::InvalidUtf8 => 4,
            EditError::InvalidSequence => 5,
        }
    }
    assert_eq!(classify(EditError::Capacity), 0);
}
