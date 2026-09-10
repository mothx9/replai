//! Revision provenance, rejection atomicity and retained host analysis.
use replai::{
    AnalysisOutcome::{Applied, Stale},
    AnalysisSnapshot, EditError, Editor, Interaction,
};

#[test]
fn coherent_owned_shared_snapshot_and_cursor_invalidation() {
    fn transferable<T: Send + Sync>() {}
    transferable::<AnalysisSnapshot>();
    let mut e = Editor::new(1024, 10);
    e.insert("foo.界e\u{301}👩‍💻").unwrap();
    let s = e.analysis_snapshot();
    let shared = s.clone();
    assert_eq!(s.text().as_ptr(), shared.text().as_ptr());
    assert_eq!((s.revision(), s.cursor()), (e.revision(), e.cursor()));
    e.left();
    assert_ne!(s.revision(), e.revision());
    assert_eq!(s.text(), "foo.界e\u{301}👩‍💻");
    let state = e.analysis_snapshot();
    assert_eq!(e.replace_at(s.revision(), 0..0, "stale"), Ok(Stale));
    assert_eq!(
        (e.text(), e.cursor(), e.revision()),
        (state.text(), state.cursor(), state.revision())
    );
    assert_eq!(e.replace_at(state.revision(), 0..3, "bar"), Ok(Applied));
    assert_eq!(e.text(), "bar.界e\u{301}👩‍💻");
}

#[test]
fn noops_rejections_and_history_admission_preserve_identity() {
    let mut e = Editor::new(8, 2);
    let empty = e.revision();
    e.left();
    e.home();
    e.end();
    e.right();
    e.backspace();
    e.delete();
    e.clear();
    e.history_up();
    e.history_down();
    e.admit_history("entry").unwrap();
    assert_eq!(e.revision(), empty);
    e.insert("e\u{301}").unwrap();
    let r = e.revision();
    e.end();
    e.right();
    e.delete();
    e.insert("").unwrap();
    assert_eq!(e.replace(0..3, "e\u{301}"), Ok(()));
    assert_eq!(e.replace(1..3, "x"), Err(EditError::InvalidRange));
    assert_eq!(e.replace(0..3, "\x1b"), Err(EditError::InvalidText));
    assert_eq!(e.insert("too long"), Err(EditError::Capacity));
    assert_eq!(e.revision(), r);
    assert_eq!(e.replace_at(r, 0..3, "yes"), Ok(Applied));
}

#[test]
fn identical_later_draft_never_revives_analysis_or_changes_history_on_stale() {
    let mut e = Editor::new(32, 3);
    e.insert("draft").unwrap();
    e.left();
    e.admit_history("old").unwrap();
    let original = e.analysis_snapshot();
    e.history_up();
    let recalled = e.analysis_snapshot();
    e.history_down();
    assert_eq!((e.text(), e.cursor()), (original.text(), original.cursor()));
    assert_ne!(e.revision(), original.revision());
    assert_eq!(
        e.replace_at(original.revision(), 0..usize::MAX, "\x1b"),
        Ok(Stale)
    );
    e.history_up();
    assert_eq!(e.replace_at(recalled.revision(), 0..3, "wrong"), Ok(Stale));
    e.history_down();
    assert_eq!((e.text(), e.cursor()), (original.text(), original.cursor()));
    e.clear();
    e.insert(original.text()).unwrap();
    e.left();
    assert_eq!(e.replace_at(original.revision(), 0..5, "wrong"), Ok(Stale));
}

#[test]
fn generated_mutation_oracle_tracks_exact_visible_state() {
    let mut e = Editor::new(128, 4);
    for h in ["history", "界e\u{301}", "👩‍💻"] {
        e.admit_history(h).unwrap();
    }
    let mut seed = 71u64;
    for _ in 0..20_000 {
        let before = e.analysis_snapshot();
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        match (seed >> 32) % 15 {
            0 => {
                let _ = e.insert("a");
            }
            1 => {
                let _ = e.insert("界");
            }
            2 => {
                let _ = e.insert("e\u{301}");
            }
            3 => e.left(),
            4 => e.right(),
            5 => e.home(),
            6 => e.end(),
            7 => e.backspace(),
            8 => e.delete(),
            9 => e.history_up(),
            10 => e.history_down(),
            11 => e.clear(),
            12 => {
                let _ = e.replace(0..e.text().len(), "paste\n界");
            }
            13 => {
                assert_eq!(e.insert("\x1b"), Err(EditError::InvalidText));
            }
            _ => {
                let _ = e.replace(e.cursor()..e.cursor(), "");
            }
        }
        let changed = (before.text(), before.cursor()) != (e.text(), e.cursor());
        assert_eq!(before.revision() != e.revision(), changed);
        if changed {
            let now = e.analysis_snapshot();
            assert_eq!(e.replace_at(before.revision(), 0..0, "stale"), Ok(Stale));
            assert_eq!(
                (e.text(), e.cursor(), e.revision()),
                (now.text(), now.cursor(), now.revision())
            );
        }
    }
}

#[test]
fn one_host_parse_multiple_derivations_and_closed_interaction() {
    let mut i = Interaction::new(Editor::new(128, 0));
    i.editor_mut().unwrap().insert("run task").unwrap();
    let snapshot = i.analysis_snapshot();
    // These are fixture-owned derivations, not completion/highlight/validation APIs.
    let words: Vec<_> = snapshot.text().split_whitespace().collect();
    let prefix = words.last().copied().unwrap();
    let ranges = vec![0..words[0].len(), 4..snapshot.text().len()];
    let complete = words.len() == 2;
    assert_eq!((prefix, ranges, complete), ("task", vec![0..3, 4..8], true));
    assert_eq!(snapshot.revision(), i.revision());
    i.editor_mut().unwrap().insert("!").unwrap();
    assert_ne!(snapshot.revision(), i.revision());
    assert_eq!(snapshot.text(), "run task");
}
