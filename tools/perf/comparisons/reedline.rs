//! Minimal memory history and one fixed completion; no filesystem discovery.
#[path = "../exit_gate.rs"]
mod exit_gate;
use reedline::{
    ColumnarMenu, Completer, CompletionResult, DefaultPrompt, DefaultPromptSegment, Emacs,
    FileBackedHistory, History, HistoryItem, KeyCode, KeyModifiers, MenuBuilder, Reedline,
    ReedlineEvent, ReedlineMenu, Signal, Span, Suggestion, default_emacs_keybindings,
};
struct Fixed;
impl Completer for Fixed {
    fn complete(&mut self, _: &str, pos: usize) -> CompletionResult {
        CompletionResult::fresh(vec![Suggestion {
            value: "replacement".into(),
            span: Span::new(0, pos),
            ..Suggestion::default()
        }])
    }
}
fn main() {
    let _exit_gate = exit_gate::Gate::new();
    let mut history = FileBackedHistory::new(100).unwrap();
    history
        .save(HistoryItem::from_command_line("history first"))
        .unwrap();
    history
        .save(HistoryItem::from_command_line("history second"))
        .unwrap();
    let mut keys = default_emacs_keybindings();
    keys.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".into()),
            ReedlineEvent::MenuNext,
        ]),
    );
    let mut editor = Reedline::create()
        .with_edit_mode(Box::new(Emacs::new(keys)))
        .with_history(Box::new(history))
        .with_ansi_colors(false)
        .with_completer(Box::new(Fixed))
        .with_quick_completions(true)
        .with_menu(ReedlineMenu::EngineCompleter(Box::new(
            ColumnarMenu::default().with_name("completion_menu"),
        )));
    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic("p".into()),
        DefaultPromptSegment::Empty,
    );
    if let Ok(Signal::Success(text)) = editor.read_line(&prompt) {
        use std::io::Write;
        let mut receipt = std::fs::OpenOptions::new()
            .write(true)
            .open(format!(
                "/dev/fd/{}",
                std::env::var("P0_RECEIPT_FD").unwrap()
            ))
            .unwrap();
        writeln!(
            receipt,
            "SUBMITTED:{}",
            text.as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        )
        .unwrap();
    }
}
