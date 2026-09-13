//! Live-process heap oracle for configurable completion surfaces on macOS.
use replai::{
    Action, CompletionAction, CompletionItem, EditAction, Editor, Key, KeyMap, MatchCase,
    NamedKey, complete_fuzzy, complete_prefix, suggest_from_history,
};
use std::io::{Read, Write};

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
        "READY prefix={} fuzzy={} suggestion={} custom_keys={}",
        prefix.candidates().len(),
        fuzzy.candidates().len(),
        suggestion.text(),
        keys.custom_len()
    );
    std::io::stdout().flush()?;
    std::io::stdin().read_exact(&mut [0_u8])?;

    std::hint::black_box((keys, editor, prefix, fuzzy, suggestion));
    Ok(())
}
