use super::Engine;
use crate::actions::Input;
use crate::{
    Action, AnalysisOutcome, CompletionAction, CompletionCandidate, CompletionSet, EditAction,
    Editor, Prompt, Suggestion, SuggestionAction,
};

fn engine(text: &str) -> Engine {
    let mut engine = Engine::new(Editor::new(256, 4));
    engine.editor.insert(text).unwrap();
    engine
        .start(Prompt::new("test").unwrap(), (40, 12))
        .unwrap();
    engine
}

#[test]
fn stale_delivery_is_silent_and_acceptance_is_one_fresh_undoable_edit() {
    let mut e = engine("dep");
    let old = e.editor.analysis_snapshot();
    e.apply(Input::Text("l".into())).unwrap();
    let before = format!("{:?}", e.editor);
    let (outcome, effects) = e
        .present_suggestion(Suggestion::new(old.revision(), "oy").unwrap())
        .unwrap();
    assert_eq!(outcome, AnalysisOutcome::Stale);
    assert!(effects.mutations.is_empty());
    assert_eq!(format!("{:?}", e.editor), before);
    let revision = e.editor.revision();
    e.present_suggestion(Suggestion::new(revision, "oy").unwrap())
        .unwrap();
    e.apply(Action::Suggestion(SuggestionAction::Accept).into())
        .unwrap();
    assert_eq!(e.editor.text(), "deploy");
    assert_ne!(e.editor.revision(), revision);
    e.apply(Input::Edit(EditAction::Undo)).unwrap();
    assert_eq!(e.editor.text(), "depl");
    assert_ne!(e.editor.revision(), revision);
    e.apply(Input::Edit(EditAction::Redo)).unwrap();
    assert_eq!(e.editor.text(), "deploy");
}

#[test]
fn dismissal_resize_output_and_precedence_preserve_canonical_state() {
    let mut e = engine("run");
    let revision = e.editor.revision();
    e.present_suggestion(Suggestion::new(revision, " task").unwrap())
        .unwrap();
    e.apply(Input::Resize(20, 8)).unwrap();
    e.external_output(crate::Role::Default, "notice").unwrap();
    assert!(e.suggestion().is_some());
    let candidate = CompletionCandidate::new(0..3, "runner", "runner").unwrap();
    e.present_completions(CompletionSet::new(revision, vec![candidate]).unwrap())
        .unwrap();
    assert!(e.suggestion().is_some());
    e.completion_action(CompletionAction::Dismiss).unwrap();
    assert!(e.suggestion().is_some());
    e.apply(Action::Suggestion(SuggestionAction::Dismiss).into())
        .unwrap();
    assert!(e.suggestion().is_none());
    assert_eq!(e.editor.text(), "run");
    assert_eq!(e.editor.revision(), revision);
}

#[test]
fn right_at_end_accepts_but_right_inside_only_moves() {
    let mut e = engine("go");
    let revision = e.editor.revision();
    e.present_suggestion(Suggestion::new(revision, "od").unwrap())
        .unwrap();
    e.apply(Input::Edit(EditAction::Right)).unwrap();
    assert_eq!(e.editor.text(), "good");
    e.apply(Input::Edit(EditAction::Undo)).unwrap();
    e.apply(Input::Edit(EditAction::Left)).unwrap();
    let revision = e.editor.revision();
    assert!(matches!(
        e.present_suggestion(Suggestion::new(revision, "x").unwrap()),
        Err(crate::SuggestionError::NotAtEnd)
    ));
}

#[test]
fn page_navigation_clamps_and_resize_keeps_identity() {
    let mut e = engine("x");
    let revision = e.editor.revision();
    let candidates = (0..100)
        .map(|i| CompletionCandidate::new(0..1, &format!("x{i}"), &format!("item {i}")).unwrap())
        .collect();
    e.present_completions(CompletionSet::new(revision, candidates).unwrap())
        .unwrap();
    e.completion_action(CompletionAction::PageNext).unwrap();
    let selected = e.completion_selection().unwrap();
    assert!(selected.index > 0);
    e.apply(Input::Resize(20, 4)).unwrap();
    assert_eq!(e.completion_selection().unwrap().index, selected.index);
    e.completion_action(CompletionAction::Last).unwrap();
    assert_eq!(e.completion_selection().unwrap().index, 99);
    e.completion_action(CompletionAction::PageNext).unwrap();
    assert_eq!(e.completion_selection().unwrap().index, 99);
    e.completion_action(CompletionAction::First).unwrap();
    assert_eq!(e.completion_selection().unwrap().index, 0);
    e.completion_action(CompletionAction::PagePrevious).unwrap();
    assert_eq!(e.completion_selection().unwrap().index, 0);
}
