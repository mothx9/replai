use super::*;
use crate::{
    AnalysisOutcome as A, CompletionAction as C, CompletionCandidate as Candidate, CompletionError,
    CompletionSet, EditError,
};
fn engine() -> Engine {
    let mut editor = Editor::new(1024, 10);
    editor.insert("bu").unwrap();
    editor.admit_history("older").unwrap();
    let mut e = Engine::new(editor);
    e.start(Prompt::new("demo").unwrap(), (80, 24)).unwrap();
    e
}
fn set(e: &Engine, count: usize) -> CompletionSet {
    CompletionSet::new(
        e.editor.revision(),
        (0..count)
            .map(|i| {
                Candidate::new(
                    0..e.editor.text().len(),
                    &format!("build-{i} "),
                    &format!("build-{i}"),
                )
                .unwrap()
                .with_annotation("Build a project 界 👩‍💻")
                .unwrap()
            })
            .collect(),
    )
    .unwrap()
}
fn display(effects: &Effects) -> String {
    effects
        .mutations
        .iter()
        .filter_map(|m| {
            if let Mutation::Text(t) = m {
                Some(t.as_str())
            } else {
                None
            }
        })
        .collect()
}
#[test]
fn selection_is_presentation_and_acceptance_is_one_existing_edit() {
    let mut e = engine();
    let before = e.editor.analysis_snapshot();
    let effects = e.present_completions(set(&e, 3)).unwrap().1;
    assert!(display(&effects).contains("> build-0"));
    for i in 1..=12 {
        e.apply(Input::Request(Request::Completion)).unwrap();
        assert_eq!(e.completion_selection().unwrap().index, i % 3);
        assert_eq!(e.editor.revision(), before.revision());
        assert_eq!(
            (e.editor.text(), e.editor.cursor()),
            (before.text(), before.cursor())
        );
    }
    e.apply(Input::Request(Request::CompletionPrevious))
        .unwrap();
    assert_eq!(e.completion_selection().unwrap().index, 2);
    let effects = e.apply(Input::Request(Request::Submit)).unwrap();
    assert!(effects.event.is_none());
    assert_eq!(e.editor.text(), "build-2 ");
    assert_ne!(e.editor.revision(), before.revision());
    assert!(e.completion_selection().is_none());
    assert!(matches!(
        e.apply(Input::Request(Request::Submit)).unwrap().event,
        Some(Event::Submitted(_))
    ));
}
#[test]
fn all_candidates_validate_atomically_and_stale_has_no_effects() {
    let mut e = engine();
    e.present_completions(set(&e, 3)).unwrap();
    let selected = e.completion_selection();
    for bad in [
        Candidate::new(0..3, "x", "bad").unwrap(),
        Candidate::new(0..2, &"x".repeat(1025), "large").unwrap(),
    ] {
        let result = CompletionSet::new(
            e.editor.revision(),
            vec![Candidate::new(0..2, "valid", "valid").unwrap(), bad],
        )
        .unwrap();
        assert!(matches!(
            e.present_completions(result),
            Err(CompletionError::Edit(_))
        ));
        assert_eq!(e.completion_selection(), selected);
        assert_eq!(e.editor.text(), "bu");
    }
    let old = set(&e, 3);
    e.apply(Input::Edit(EditCommand::Left)).unwrap();
    let current = e.editor.analysis_snapshot();
    let (outcome, effects) = e.present_completions(old).unwrap();
    assert_eq!(outcome, A::Stale);
    assert!(effects.mutations.is_empty() && effects.event.is_none());
    assert_eq!(e.editor.revision(), current.revision());
    assert!(e.completion_selection().is_none());
}
#[test]
fn same_revision_delivery_is_explicit_replacement_not_job_ranking() {
    let mut e = engine();
    let a = set(&e, 2);
    let b = set(&e, 3);
    e.present_completions(b).unwrap();
    e.completion_action(C::Next).unwrap();
    e.present_completions(a).unwrap();
    assert_eq!(
        (
            e.completion_selection().unwrap().index,
            e.completion_selection().unwrap().count
        ),
        (0, 2)
    );
}
#[test]
fn zero_single_duplicate_and_noop_acceptance_are_deliberate() {
    let mut e = engine();
    let revision = e.editor.revision();
    e.present_completions(set(&e, 1)).unwrap();
    assert_eq!(e.editor.revision(), revision);
    e.present_completions(set(&e, 0)).unwrap();
    assert!(e.completion_selection().is_none());
    let c = Candidate::new(0..2, "bu", "same").unwrap();
    e.present_completions(CompletionSet::new(revision, vec![c.clone(), c]).unwrap())
        .unwrap();
    e.completion_action(C::Next).unwrap();
    e.completion_action(C::Accept).unwrap();
    assert_eq!(e.editor.revision(), revision); // I0 no-op semantics remain authoritative.
    assert!(e.completion_selection().is_none());
}
#[test]
fn output_resize_noop_and_rejection_keep_current_selection() {
    let mut e = engine();
    e.apply(Input::Edit(EditCommand::Home)).unwrap();
    e.present_completions(set(&e, 10)).unwrap();
    e.completion_action(C::Previous).unwrap();
    let selected = e.completion_selection();
    for input in [
        Input::Edit(EditCommand::Left),
        Input::Resize(20, 8),
        Input::Request(Request::Redraw),
        Input::Rejected(EditError::InvalidUtf8),
    ] {
        e.apply(input).unwrap();
        assert_eq!(e.completion_selection(), selected);
    }
    e.external_output(Role::Warning, "notice").unwrap();
    assert_eq!(e.completion_selection(), selected);
    e.output_document(
        &crate::Document::new(vec![crate::document::Block::Paragraph(
            crate::Text::new("record").unwrap(),
        )])
        .unwrap(),
    )
    .unwrap();
    assert_eq!(e.completion_selection(), selected);
    e.completion_action(C::Accept).unwrap();
    assert_eq!(e.editor.text(), "build-9 ");
}
#[test]
fn edits_history_paste_and_direct_completion_invalidate() {
    for input in [
        Input::Text("x".into()),
        Input::Edit(EditCommand::Left),
        Input::Edit(EditCommand::Home),
        Input::Edit(EditCommand::Backspace),
        Input::Edit(EditCommand::HistoryPrevious),
        Input::Text("a\n界\t".into()),
    ] {
        let mut e = engine();
        e.present_completions(set(&e, 2)).unwrap();
        let old = e.editor.revision();
        e.apply(input).unwrap();
        assert_ne!(e.editor.revision(), old);
        assert!(e.completion_selection().is_none());
    }
    let mut e = engine();
    e.present_completions(set(&e, 2)).unwrap();
    e.complete_at(e.editor.revision(), 0..2, "direct").unwrap();
    assert!(e.completion_selection().is_none());
}
#[test]
fn close_and_finish_erase_transient_rows_before_leaving() {
    for finish in [false, true] {
        let mut e = engine();
        let revision = e.editor.revision();
        e.present_completions(set(&e, 100)).unwrap();
        let effects = if finish {
            e.apply(Input::Request(Request::Interrupt)).unwrap()
        } else {
            e.close()
        };
        assert!(!display(&effects).contains("build-"));
        assert!(effects.mutations.contains(&Mutation::ClearLine));
        assert!(e.completion_selection().is_none());
        assert_eq!(e.editor.revision() == revision, !finish);
        e.start(Prompt::new("again").unwrap(), (80, 24)).unwrap();
        assert!(e.completion_selection().is_none());
    }
}
#[test]
fn viewport_width_and_unicode_are_bounded_and_ordered() {
    for width in [2, 20, 40, 80, 132] {
        for rows in [2, 3, 8, 24] {
            let mut e = engine();
            e.apply(Input::Resize(width, rows)).unwrap();
            let candidates = (0..1000)
                .map(|i| {
                    Candidate::new(
                        0..2,
                        "insert",
                        &format!("{i}:界e\u{301}👩‍💻{}", "界".repeat(20)),
                    )
                    .unwrap()
                    .with_annotation("explanation")
                    .unwrap()
                })
                .collect();
            e.present_completions(CompletionSet::new(e.editor.revision(), candidates).unwrap())
                .unwrap();
            for _ in 0..12 {
                let s = e.surface.as_ref().unwrap();
                let c = s.completion.as_ref().unwrap();
                let frame = c.frame(&e.editor, &s.prompt, s.size, None);
                assert!(
                    frame.lines.len() <= rows,
                    "{width}x{rows}: {} rows",
                    frame.lines.len()
                );
                assert!(frame.widths.iter().all(|w| *w <= width));
                assert!(frame.cursor.row < frame.lines.len());
                e.completion_action(C::Previous).unwrap();
            }
        }
    }
}
#[test]
fn generated_interleaving_never_revives_a_stale_set() {
    let mut e = engine();
    let mut seed = 7u64;
    let mut old = set(&e, 3);
    for step in 0..4000 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        match (seed >> 32) % 7 {
            0 => {
                old = set(&e, 3);
                e.present_completions(old.clone()).unwrap();
            }
            1 => {
                e.apply(Input::Edit(EditCommand::Left)).unwrap();
            }
            2 => {
                e.apply(Input::Edit(EditCommand::Right)).unwrap();
            }
            3 => {
                e.apply(Input::Text("x".into())).unwrap();
            }
            4 => {
                e.apply(Input::Edit(EditCommand::Backspace)).unwrap();
            }
            5 if e.completion_selection().is_some() => {
                let r = e.editor.revision();
                e.completion_action(C::Next).unwrap();
                assert_eq!(e.editor.revision(), r);
            }
            _ => {
                let before = format!("{:?}", e.editor);
                let stale = old.revision() != e.editor.revision();
                let (a, effects) = e.present_completions(old.clone()).unwrap();
                if stale {
                    assert_eq!(a, A::Stale);
                    assert!(effects.mutations.is_empty());
                    assert_eq!(format!("{:?}", e.editor), before, "step {step}");
                }
            }
        }
        if let Some(selection) = e.completion_selection() {
            assert_eq!(selection.revision, e.editor.revision());
        }
    }
}

#[test]
fn per_candidate_ranges_and_joining_graphemes_use_editor_validation() {
    let mut e = engine();
    e.complete(0..2, "src/co tail").unwrap();
    let revision = e.editor.revision();
    let candidates = vec![
        Candidate::new(0..6, "src/core.rs", "core.rs").unwrap(),
        Candidate::new(0..11, "all", "whole draft").unwrap(),
    ];
    e.present_completions(CompletionSet::new(revision, candidates).unwrap())
        .unwrap();
    e.completion_action(C::Accept).unwrap();
    assert_eq!(e.editor.text(), "src/core.rs tail");
    e.complete(0..e.editor.text().len(), "e\u{301}x").unwrap();
    e.present_completions(set(&e, 2)).unwrap();
    let selection = e.completion_selection();
    let invalid = CompletionSet::new(
        e.editor.revision(),
        vec![Candidate::new(0..1, "a", "split grapheme").unwrap()],
    )
    .unwrap();
    assert!(matches!(
        e.present_completions(invalid),
        Err(CompletionError::Edit(EditError::InvalidRange))
    ));
    assert_eq!(e.completion_selection(), selection);
    let merging = CompletionSet::new(
        e.editor.revision(),
        vec![Candidate::new(3..3, "\u{301}", "join mark").unwrap()],
    )
    .unwrap();
    e.present_completions(merging).unwrap();
    e.completion_action(C::Accept).unwrap();
    assert_eq!(e.editor.text(), "e\u{301}\u{301}x");
    assert_eq!(e.editor.cursor(), 5);
}
