use crate::{
    AnalysisOutcome, AnalysisPresentation, AnalysisSpan, CompletionCandidate, CompletionSet,
    Editor, HistorySearchLimits, HistorySearchSource, Prompt, Role,
    actions::{EditCommand as E, Input, Request as R},
    engine::Engine,
};

#[test]
fn accepting_identical_history_text_still_creates_a_fresh_identity() {
    let mut engine = crate::engine::Engine::new(Editor::new(64, 2));
    engine.editor.insert("same").unwrap();
    engine
        .set_history_source(Some(
            HistorySearchSource::from_entries(
                ["same"],
                HistorySearchLimits::new(2, 64, 16).unwrap(),
            )
            .unwrap(),
        ))
        .unwrap();
    engine.start(Prompt::new("q").unwrap(), (40, 8)).unwrap();
    let before = engine.editor.revision();
    engine.apply(Input::Edit(E::HistorySearchOlder)).unwrap();
    engine
        .apply(Input::Request(crate::actions::Request::Submit))
        .unwrap();
    assert_ne!(engine.editor.revision(), before);
    assert_eq!((engine.editor.text(), engine.editor.cursor()), ("same", 4));
}

fn opened() -> Engine {
    let mut engine = Engine::new(Editor::new(256, 8));
    engine.editor.admit_history("older alpha").unwrap();
    engine.editor.admit_history("newest beta").unwrap();
    let source = HistorySearchSource::from_entries(
        ["external gamma", "external delta"],
        HistorySearchLimits::new(8, 256, 32).unwrap(),
    )
    .unwrap();
    engine.set_history_source(Some(source)).unwrap();
    engine.start(Prompt::new("test").unwrap(), (40, 8)).unwrap();
    engine
}

#[test]
fn reverse_search_is_noncanonical_until_atomic_accept() {
    let mut engine = opened();
    engine.apply(Input::Text("draft".into())).unwrap();
    engine.apply(Input::Edit(E::Left)).unwrap();
    let original = engine.editor.analysis_snapshot();
    engine.apply(Input::Edit(E::HistorySearchOlder)).unwrap();
    assert_eq!(engine.history_search_match(), Some("newest beta"));
    engine.apply(Input::Text("alpha".into())).unwrap();
    assert_eq!(engine.history_search_query(), Some("alpha"));
    assert_eq!(engine.history_search_match(), Some("older alpha"));
    assert_eq!(
        (
            engine.editor.text(),
            engine.editor.cursor(),
            engine.editor.revision()
        ),
        (original.text(), original.cursor(), original.revision())
    );
    engine.apply(Input::Request(R::Submit)).unwrap();
    assert_eq!(engine.editor.text(), "older alpha");
    assert_eq!(engine.editor.cursor(), "older alpha".len());
    assert_ne!(engine.editor.revision(), original.revision());
    assert!(engine.editor.undo());
    assert_eq!((engine.editor.text(), engine.editor.cursor()), ("draft", 4));
}

#[test]
fn search_dismiss_resize_output_and_query_bounds_preserve_original() {
    let mut engine = opened();
    engine.apply(Input::Text("draft界".into())).unwrap();
    engine.apply(Input::Edit(E::Left)).unwrap();
    let original = engine.editor.analysis_snapshot();
    engine.apply(Input::Edit(E::HistorySearchOlder)).unwrap();
    engine.apply(Input::Resize(20, 5)).unwrap();
    engine
        .external_output(Role::Success, "host notice")
        .unwrap();
    assert_eq!(engine.history_search_match(), Some("newest beta"));
    let rejected = engine.apply(Input::Text("x".repeat(33))).unwrap();
    assert!(matches!(rejected.event, Some(crate::Event::Rejected(_))));
    assert_eq!(engine.history_search_query(), Some(""));
    engine
        .apply(Input::Request(R::CompletionAction(
            crate::CompletionAction::Dismiss,
        )))
        .unwrap();
    assert_eq!(
        (
            engine.editor.text(),
            engine.editor.cursor(),
            engine.editor.revision()
        ),
        (original.text(), original.cursor(), original.revision())
    );
}

#[test]
fn search_and_completion_selection_are_mutually_exclusive_but_analysis_is_safe() {
    let mut engine = opened();
    engine.apply(Input::Text("ne".into())).unwrap();
    let revision = engine.editor.revision();
    let presentation = AnalysisPresentation::new(
        revision,
        vec![AnalysisSpan::new(0..2, Role::Accent).unwrap()],
        None,
    )
    .unwrap();
    engine.present_analysis(presentation).unwrap();
    engine.apply(Input::Edit(E::HistorySearchOlder)).unwrap();
    let candidate = CompletionCandidate::new(0..2, "new", "new").unwrap();
    assert!(
        engine
            .present_completions(CompletionSet::new(revision, vec![candidate]).unwrap())
            .is_err()
    );
    assert_eq!(engine.history_search_query(), Some(""));
    engine
        .apply(Input::Request(R::CompletionAction(
            crate::CompletionAction::Dismiss,
        )))
        .unwrap();
    assert!(engine.analysis_presentation().is_some());
    engine.apply(Input::Text("x".into())).unwrap();
    assert!(engine.analysis_presentation().is_none());
    assert_eq!(
        engine
            .present_analysis(AnalysisPresentation::new(revision, vec![], None).unwrap())
            .unwrap()
            .0,
        AnalysisOutcome::Stale
    );
}

#[test]
fn query_refinement_cycles_duplicates_without_wrap() {
    let mut engine = opened();
    engine.apply(Input::Edit(E::HistorySearchOlder)).unwrap();
    engine.apply(Input::Text("external".into())).unwrap();
    assert_eq!(engine.history_search_match(), Some("external gamma"));
    engine.apply(Input::Edit(E::HistorySearchOlder)).unwrap();
    assert_eq!(engine.history_search_match(), Some("external delta"));
    engine.apply(Input::Edit(E::HistorySearchOlder)).unwrap();
    assert_eq!(engine.history_search_match(), None);
    engine.apply(Input::Edit(E::HistorySearchNewer)).unwrap();
    assert_eq!(engine.history_search_match(), Some("external delta"));
    engine.apply(Input::Edit(E::Backspace)).unwrap();
    assert_eq!(engine.history_search_match(), Some("external gamma"));
}
