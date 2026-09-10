//! Candidate costs are separate from ordinary editing and host discovery.
use super::*;
fn case(operation: &str, count: usize, class: &str, width: usize) -> Value {
    let mut value = spec("completion", operation, count, class, width);
    value["input_bytes"] = json!(2);
    value["candidate_count"] = json!(count);
    value
}
fn set(revision: DraftRevision, count: usize, class: &str) -> CompletionSet {
    let candidates = (0..count)
        .map(|i| {
            let label = match class {
                "unicode" => format!("{i}: 界e\u{301}👩‍💻"),
                _ => format!("build-{i}"),
            };
            let annotation = if class == "long" {
                "long explanation ".repeat(256)
            } else {
                "Build the project".into()
            };
            CompletionCandidate::new(0..2, &format!("build-{i} "), &label)
                .unwrap()
                .with_annotation(&annotation)
                .unwrap()
        })
        .collect();
    CompletionSet::new(revision, candidates).unwrap()
}
fn ready(width: usize) -> Engine {
    let mut e = Engine::new(editor("bu", 2));
    e.start(prompt(), (width, 24)).unwrap();
    e
}
pub(super) fn measure(h: &Harness) {
    for count in [1, 10, 100, 1000] {
        for class in ["short", "long", "unicode"] {
            h.measure(case("construct",count,class,80), ||Editor::new(LIMIT,0).revision(), |revision|set(*revision,count,class), |_,set|json!({"candidates":set.candidates().len(),"payload_bytes":set.candidates().iter().map(|c|c.replacement().len()+c.label().len()+c.annotation().map_or(0,str::len)).sum::<usize>(),"candidate_size":std::mem::size_of::<CompletionCandidate>(),"set_size":std::mem::size_of::<CompletionSet>(),"editor_size":std::mem::size_of::<Editor>(),"interaction_size":std::mem::size_of::<Interaction>()}));
            for width in [20, 80, 132] {
                h.measure(
                    case("validate", count, class, width),
                    || {
                        let e = ready(width);
                        let s = set(e.editor.revision(), count, class);
                        (e, s)
                    },
                    |(e, s)| s.validate(&e.editor).unwrap(),
                    |_, _| json!({"candidates_validated":count}),
                );
                h.measure(
                    case("install", count, class, width),
                    || {
                        let e = ready(width);
                        let s = set(e.editor.revision(), count, class);
                        (e, Some(s))
                    },
                    |(e, s)| e.present_completions(s.take().unwrap()).unwrap().1,
                    |(e, _), fx| {
                        assert_eq!(e.completion_selection().unwrap().count, count);
                        mutation_counts(&fx.mutations)
                    },
                );
                h.measure(
                    case("layout", count, class, width),
                    || {
                        let e = ready(width);
                        let c = completion::ActiveCompletion {
                            set: set(e.editor.revision(), count, class),
                            selected: count / 2,
                        };
                        (e, c)
                    },
                    |(e, c)| c.frame(&e.editor, &prompt(), (width, 24)),
                    |_, frame| json!({"visible_rows":frame.lines.len(),"width":frame.columns}),
                );
                for (name, action) in [
                    ("next", CompletionAction::Next),
                    ("previous", CompletionAction::Previous),
                    ("accept", CompletionAction::Accept),
                    ("dismiss", CompletionAction::Dismiss),
                ] {
                    h.measure(
                        case(name, count, class, width),
                        || {
                            let mut e = ready(width);
                            e.present_completions(set(e.editor.revision(), count, class))
                                .unwrap();
                            e
                        },
                        |e| e.completion_action(action).unwrap().1,
                        |_, fx| mutation_counts(&fx.mutations),
                    );
                }
                h.measure(
                    case("resize", count, class, width),
                    || {
                        let mut e = ready(width);
                        e.present_completions(set(e.editor.revision(), count, class))
                            .unwrap();
                        e
                    },
                    |e| e.apply(Input::Resize(width + 1, 24)).unwrap(),
                    |_, fx| mutation_counts(&fx.mutations),
                );
            }
        }
    }
}
