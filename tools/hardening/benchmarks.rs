//! Selected Q2 workloads. Preparation, verification and encoding are outside timing.
use crate::{
    actions::{EditCommand, Input, Request},
    engine::{Effects, Engine},
    input::Decoder,
    *,
};
use serde_json::json;
use std::{hint::black_box, time::Instant};
#[path = "../perf/allocation.rs"]
mod allocation;

struct State {
    e: Engine,
    text: Option<String>,
    completion: Option<CompletionSet>,
    validation: Option<ValidationResult>,
    analysis: Option<AnalysisPresentation>,
    intermediate: Option<Vec<crate::render::Mutation>>,
}
fn prepare(operation: &str, bytes: usize, lines: usize) -> State {
    let mut e = Engine::new(Editor::new(2 * 1024 * 1024, 8));
    let text = if lines > 0 {
        ("x".repeat(63) + "\n").repeat(lines)
    } else {
        "x".repeat(bytes)
    };
    e.editor.insert(&text).unwrap();
    if operation == "append" {
        // Steady append has spare capacity. Cold growth is measured separately.
        e.editor.insert("a").unwrap();
        e.editor.backspace();
    }
    e.editor.admit_history("history\nentry").unwrap();
    e.set_submission_policy(SubmissionPolicy::Validated)
        .unwrap();
    e.start(Prompt::new("q2").unwrap(), (80, 24)).unwrap();
    let candidates =
        || vec![CompletionCandidate::new(0..text.len(), "replacement", "display").unwrap(); 10];
    if matches!(operation, "menu-next" | "menu-accept") {
        e.present_completions(CompletionSet::new(e.editor.revision(), candidates()).unwrap())
            .unwrap();
    }
    if matches!(operation, "validation" | "incomplete") {
        e.apply(Input::Request(Request::Submit)).unwrap();
    }
    let revision = e.editor.revision();
    State {
        e,
        intermediate: None,
        text: Some(if operation == "burst" {
            "a".repeat(1000)
        } else {
            "\x1b[200~".to_owned() + &"a".repeat(1000) + "\x1b[201~"
        }),
        completion: Some(CompletionSet::new(revision, candidates()).unwrap()),
        validation: Some(
            ValidationResult::new(
                revision,
                if operation == "incomplete" {
                    ValidationDisposition::Incomplete
                } else {
                    ValidationDisposition::Invalid(vec![
                        Diagnostic::new("check input", None).unwrap(),
                    ])
                },
            )
            .unwrap(),
        ),
        analysis: Some(
            AnalysisPresentation::new(
                revision,
                if text.is_empty() {
                    vec![]
                } else {
                    vec![AnalysisSpan::new(0..1, Role::Accent).unwrap()]
                },
                Some(Hint::new("hint", Role::Dim).unwrap()),
            )
            .unwrap(),
        ),
    }
}
fn operation(s: &mut State, name: &str) -> Effects {
    let e = &mut s.e;
    match name {
        "append" | "append-grow" => {
            e.editor.insert("a").unwrap();
            Effects::default()
        }
        "cursor" => {
            e.editor.left();
            Effects::default()
        }
        "edit" => e.apply(Input::Text("a".into())).unwrap(),
        "vertical" => e.apply(Input::Edit(EditCommand::HistoryPrevious)).unwrap(),
        "history" => {
            e.editor.history_up();
            Effects::default()
        }
        "replace" => {
            e.complete_at(e.editor.revision(), 0..e.editor.text().len(), "replacement")
                .unwrap()
                .1
        }
        "menu-show" => {
            e.present_completions(s.completion.take().unwrap())
                .unwrap()
                .1
        }
        "menu-next" => e.completion_action(CompletionAction::Next).unwrap().1,
        "menu-accept" => e.completion_action(CompletionAction::Accept).unwrap().1,
        "validation" | "incomplete" => e.apply_validation(s.validation.take().unwrap()).unwrap().1,
        "analysis" => e.present_analysis(s.analysis.take().unwrap()).unwrap().1,
        "output" => e
            .external_output(Role::Default, "finite serialized host output")
            .unwrap(),
        "resize" => e.apply(Input::Resize(40, 12)).unwrap(),
        "burst" => {
            for b in s.text.take().unwrap().bytes() {
                e.apply_deferred(Input::Text(char::from(b).to_string()))
                    .unwrap();
            }
            let request = e.apply(Input::Request(Request::Submit)).unwrap();
            s.intermediate = Some(request.mutations);
            drop(request.event);
            e.apply_validation(
                ValidationResult::new(e.editor.revision(), ValidationDisposition::Complete)
                    .unwrap(),
            )
            .unwrap()
            .1
        }
        "paste" => {
            let mut d = Decoder::new(4096);
            for b in s.text.take().unwrap().bytes() {
                if let Some(k) = d.feed(b) {
                    e.apply_deferred(crate::keymap::binding(k).unwrap())
                        .unwrap();
                }
            }
            e.flush()
        }
        _ => panic!("unknown operation {name}"),
    }
}
pub fn benchmark(samples: usize, batches: usize) {
    assert!((1..=1000).contains(&samples) && (1..=100).contains(&batches));
    let mut workloads = vec![];
    for name in [
        "append",
        "append-grow",
        "cursor",
        "burst",
        "paste",
        "history",
        "replace",
        "menu-show",
        "menu-next",
        "menu-accept",
        "validation",
        "incomplete",
        "analysis",
        "output",
        "resize",
    ] {
        workloads.push((name, if name == "burst" { 0 } else { 1024 }, 0));
    }
    for lines in [10, 100, 1000] {
        for name in ["edit", "vertical", "resize"] {
            workloads.push((name, lines * 64, lines));
        }
    }
    for bytes in [65536, 1048576] {
        for name in ["edit", "cursor", "analysis", "resize"] {
            workloads.push((name, bytes, 0));
        }
    }
    let mut resolution = u128::MAX;
    for _ in 0..10000 {
        let t = Instant::now();
        let dt = t.elapsed().as_nanos();
        if dt > 0 {
            resolution = resolution.min(dt);
        }
    }
    println!(
        "{}",
        json!({"timer_resolution_ns":resolution,"instrumented_allocations":cfg!(feature="allocations"),"samples":samples,"batches":batches})
    );
    for (name, bytes, lines) in workloads {
        let mut elapsed = Vec::new();
        let mut memory = Vec::new();
        let mut vt_bytes = Vec::new();
        for _ in 0..batches {
            for _ in 0..3 {
                let mut s = prepare(name, bytes, lines);
                black_box(operation(&mut s, name));
            }
            for _ in 0..samples {
                let mut s = prepare(name, bytes, lines);
                let before = s.e.editor.analysis_snapshot();
                let counters = allocation::start();
                let started = Instant::now();
                let effects = black_box(operation(black_box(&mut s), name));
                let ns = started.elapsed().as_nanos();
                let allocated = allocation::end(counters);
                if matches!(
                    name,
                    "menu-show" | "menu-next" | "validation" | "analysis" | "output" | "resize"
                ) {
                    assert_eq!(s.e.editor.revision(), before.revision());
                }
                if name == "burst" {
                    assert_eq!(effects.event, Some(Event::Submitted("a".repeat(1000))));
                }
                if name == "menu-accept" || name == "replace" {
                    assert_eq!(s.e.editor.text(), "replacement");
                }
                let encoded =
                    crate::protocol::encode(&effects.mutations, Theme::new(false, false, None));
                let intermediate_bytes = s.intermediate.as_ref().map_or(0, |mutations| {
                    crate::protocol::encode(mutations, Theme::new(false, false, None)).len()
                });
                if name == "burst" {
                    assert!(
                        intermediate_bytes >= 1000,
                        "include the draft flush before validated submission"
                    );
                }
                vt_bytes.push(intermediate_bytes + encoded.len());
                elapsed.push(ns);
                memory.push(allocated);
            }
        }
        println!(
            "{}",
            json!({"id":format!("{name}/{bytes}/{lines}"),"bytes":bytes,"lines":lines,"columns":80,"rows":24,"samples_ns":if cfg!(feature="allocations") {None}else{Some(elapsed)},"allocations":if cfg!(feature="allocations") {Some(memory)}else{None},"encoded_bytes":vt_bytes})
        );
    }
}
