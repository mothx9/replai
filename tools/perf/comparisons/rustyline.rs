//! Minimal memory-history consumer; fixed host-selected completion, no discovery.
#[path = "../exit_gate.rs"]
mod exit_gate;
use rustyline::{
    Context, Helper, Result, completion::Completer, highlight::Highlighter, hint::Hinter,
    validate::Validator,
};
struct Fixed;
impl Helper for Fixed {}
impl Hinter for Fixed {
    type Hint = String;
}
impl Highlighter for Fixed {}
impl Validator for Fixed {}
impl Completer for Fixed {
    type Candidate = String;
    fn complete(&self, _: &str, _: usize, _: &Context<'_>) -> Result<(usize, Vec<String>)> {
        Ok((0, vec!["replacement".into()]))
    }
}
fn main() {
    let _exit_gate = exit_gate::Gate::new();
    let mut editor = rustyline::Editor::<Fixed, rustyline::history::DefaultHistory>::new().unwrap();
    editor.set_helper(Some(Fixed));
    editor.add_history_entry("history first").unwrap();
    editor.add_history_entry("history second").unwrap();
    if let Ok(text) = editor.readline("p> ") {
        eprintln!(
            "SUBMITTED:{}",
            text.as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
    }
}
