//! Submission and multiline costs, separately from ordinary editing.
use super::*;
fn ready(text: &str, position: usize, width: usize) -> Engine {
    let mut e = Engine::new(editor(text, position));
    e.set_submission_policy(SubmissionPolicy::Validated)
        .unwrap();
    e.start(prompt(), (width, 24)).unwrap();
    e
}
pub(super) fn measure(h: &Harness) {
    for lines in [10, 100, 1000] {
        let text = ("x".repeat(68) + "\n").repeat(lines);
        for width in [20, 80, 132] {
            for position in [0, text.len() / 2, text.len()] {
                for operation in [
                    "append",
                    "up",
                    "down",
                    "incomplete",
                    "invalid",
                    "complete",
                    "resize",
                    "output",
                ] {
                    let mut case = spec("validation", operation, text.len(), "multiline", width);
                    case["id"] = json!(format!("{}/pos-{position}", case["id"].as_str().unwrap()));
                    case["lines"] = json!(lines);
                    h.measure(
                        case,
                        || {
                            let mut e = ready(&text, position, width);
                            e.apply(Input::Request(Request::Submit)).unwrap();
                            e
                        },
                        |e| match operation {
                            "append" => e.apply(Input::Text("X".into())).unwrap(),
                            "up" => e
                                .apply(Input::Edit(actions::EditCommand::HistoryPrevious))
                                .unwrap(),
                            "down" => e
                                .apply(Input::Edit(actions::EditCommand::HistoryNext))
                                .unwrap(),
                            "resize" => e.apply(Input::Resize(width + 1, 24)).unwrap(),
                            "output" => e.external_output(Role::Dim, "notice").unwrap(),
                            _ => {
                                let d = match operation {
                                    "incomplete" => ValidationDisposition::Incomplete,
                                    "invalid" => ValidationDisposition::Invalid(vec![
                                        Diagnostic::new("Expected closing delimiter", Some(0..0))
                                            .unwrap(),
                                    ]),
                                    _ => ValidationDisposition::Complete,
                                };
                                e.apply_validation(
                                    ValidationResult::new(e.editor.revision(), d).unwrap(),
                                )
                                .unwrap()
                                .1
                            }
                        },
                        |_, fx| mutation_counts(&fx.mutations),
                    );
                }
            }
        }
    }
    h.measure(spec("validation","sizes",0,"none",0),||(),|_|(),|_,_|json!({"editor":std::mem::size_of::<Editor>(),"interaction":std::mem::size_of::<Interaction>(),"result":std::mem::size_of::<ValidationResult>(),"diagnostic":std::mem::size_of::<Diagnostic>(),"state":std::mem::size_of::<validation::ValidationState>()}));
}
