//! Public configuration, helper, and suggestion contract tests.
use replai::{
    Action, CompletionAction, CompletionItem, EditAction, Editor, Key, KeyMap, MAX_CUSTOM_BINDINGS,
    MatchCase, NamedKey, PathCompletionOptions, Suggestion, SuggestionAction,
    common_grapheme_prefix, complete_fuzzy, complete_path, complete_prefix, suggest_from_history,
    suggest_from_static,
};

#[test]
fn public_keymap_exposes_every_established_semantic_class() {
    let mut map = KeyMap::new();
    assert_eq!(
        map.get(Key::Named(NamedKey::Left)),
        Some(Action::Edit(EditAction::Left))
    );
    map.bind(Key::meta(b'r').unwrap(), Action::Edit(EditAction::Redo))
        .unwrap();
    map.bind(
        Key::meta(b'k').unwrap(),
        Action::Edit(EditAction::KillWordForward),
    )
    .unwrap();
    map.bind(Key::control(20).unwrap(), Action::HistorySearchAccept)
        .unwrap();
    map.bind(
        Key::Named(NamedKey::PageDown),
        Action::Completion(CompletionAction::PageNext),
    )
    .unwrap();
    map.bind(
        Key::Named(NamedKey::PageUp),
        Action::Completion(CompletionAction::PagePrevious),
    )
    .unwrap();
    map.bind(
        Key::meta(b'a').unwrap(),
        Action::Suggestion(SuggestionAction::Accept),
    )
    .unwrap();
    assert!(map.custom_len() <= MAX_CUSTOM_BINDINGS);
    assert_eq!(
        map.unbind(Key::Named(NamedKey::Tab)).unwrap(),
        Some(Action::CompletionRequest)
    );
    assert_eq!(map.get(Key::Named(NamedKey::Tab)), None);
    map.reset(Key::Named(NamedKey::Tab)).unwrap();
    assert_eq!(
        map.get(Key::Named(NamedKey::Tab)),
        Some(Action::CompletionRequest)
    );
    map.reset_all();
    assert!(!map.is_customized());
}

#[test]
fn helper_outputs_are_revision_bound_safe_and_ordered() {
    let mut editor = Editor::new(1024, 4);
    editor.insert("dep").unwrap();
    let snapshot = editor.analysis_snapshot();
    let items = [
        CompletionItem::new("deploy").unwrap(),
        CompletionItem::new("depend").unwrap(),
        CompletionItem::new("debug").unwrap(),
    ];
    let set = complete_prefix(&snapshot, 0..3, "dep", &items, MatchCase::Sensitive).unwrap();
    assert_eq!(
        set.candidates()
            .iter()
            .map(|c| c.replacement())
            .collect::<Vec<_>>(),
        ["deploy", "depend"]
    );
    let fuzzy = complete_fuzzy(&snapshot, 0..3, "dg", &items, MatchCase::Sensitive).unwrap();
    assert_eq!(fuzzy.candidates()[0].replacement(), "debug");
    assert_eq!(common_grapheme_prefix(&["界面", "界線"]), "界");
    let suggestion = suggest_from_static(&snapshot, &["deploy", "debug"])
        .unwrap()
        .unwrap();
    assert_eq!(suggestion.text(), "loy");
    assert_eq!(
        suggest_from_history(&snapshot, &["depend", "deploy"])
            .unwrap()
            .unwrap()
            .text(),
        "end"
    );
    editor.insert("x").unwrap();
    assert_ne!(suggestion.revision(), editor.revision());
}

#[test]
fn path_completion_is_one_level_sorted_and_does_not_quote() {
    let root = std::env::temp_dir().join(format!("replai-helper-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("alpha dir")).unwrap();
    std::fs::write(root.join("alpine"), b"x").unwrap();
    std::fs::write(root.join("beta"), b"x").unwrap();
    let editor = Editor::new(1024, 0);
    let snapshot = editor.analysis_snapshot();
    let options = PathCompletionOptions::new(&root);
    let set = complete_path(&snapshot, 0..0, "al", &options).unwrap();
    let values = set
        .candidates()
        .iter()
        .map(|c| c.replacement())
        .collect::<Vec<_>>();
    assert_eq!(
        values,
        [
            format!("alpha dir{}", std::path::MAIN_SEPARATOR),
            "alpine".to_owned()
        ]
    );
    assert!(!values[0].contains('\"'));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn suggestion_constructor_rejects_controls_and_is_noncanonical() {
    let editor = Editor::new(32, 0);
    let revision = editor.revision();
    let suggestion = Suggestion::new(revision, "loy").unwrap();
    assert_eq!(suggestion.text(), "loy");
    assert_eq!(editor.text(), "");
    assert!(Suggestion::new(revision, "\u{1b}[31m").is_err());
}
