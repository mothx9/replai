//! Portable construction example for configurable actions, generic completion and suggestions.
use replai::{
    Action, CompletionAction, CompletionItem, EditAction, Editor, Key, KeyMap, MatchCase, NamedKey,
    complete_fuzzy, complete_prefix, suggest_from_history,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut keys = KeyMap::new();
    keys.bind(Key::meta(b'r')?, Action::Edit(EditAction::Redo))?;
    keys.bind(
        Key::Named(NamedKey::PageDown),
        Action::Completion(CompletionAction::PageNext),
    )?;
    let mut editor = Editor::new(1024, 20);
    editor.insert("dep")?;
    let snapshot = editor.analysis_snapshot();
    let commands = [
        CompletionItem::new("deploy")?,
        CompletionItem::new("describe")?,
        CompletionItem::new("debug")?,
    ];
    let prefix = complete_prefix(&snapshot, 0..3, "dep", &commands, MatchCase::Sensitive)?;
    let fuzzy = complete_fuzzy(&snapshot, 0..3, "dg", &commands, MatchCase::Sensitive)?;
    let suggestion = suggest_from_history(&snapshot, &["deploy service", "debug task"])?.unwrap();
    println!(
        "prefix={} fuzzy={} suggestion={} custom_keys={}",
        prefix.candidates().len(),
        fuzzy.candidates().len(),
        suggestion.text(),
        keys.custom_len()
    );
    Ok(())
}
