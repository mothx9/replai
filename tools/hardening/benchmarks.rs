//! Selected Q2 workloads. Preparation, verification and encoding are outside timing.
use crate::{
    actions::{EditCommand, Input, Request},
    engine::{Effects, Engine},
    input::Decoder,
    *,
};
use serde_json::json;
use std::{hint::black_box, sync::OnceLock, time::Instant};
#[path = "../perf/allocation.rs"]
mod allocation;

struct State {
    e: Engine,
    text: Option<String>,
    completion: Option<CompletionSet>,
    validation: Option<ValidationResult>,
    analysis: Option<AnalysisPresentation>,
    intermediate: Option<Vec<crate::render::Mutation>>,
    keymap: KeyMap,
    items: Vec<CompletionItem>,
    suggestion: Option<Suggestion>,
    path: Option<std::path::PathBuf>,
}
fn path_fixture(item_count: usize) -> std::path::PathBuf {
    static SMALL: OnceLock<std::path::PathBuf> = OnceLock::new();
    static LARGE: OnceLock<std::path::PathBuf> = OnceLock::new();
    let slot = if item_count == 1_000 { &LARGE } else { &SMALL };
    slot.get_or_init(|| {
        let root = std::env::temp_dir().join(format!("replai-q2-{}-{item_count}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        for i in 0..item_count {
            std::fs::write(root.join(format!("alpha-{i:04}")), b"x").unwrap();
        }
        root
    }).clone()
}
fn prepare(operation: &str, bytes: usize, lines: usize) -> State {
    let mut e = Engine::new(Editor::new(2 * 1024 * 1024, 1_024));
    let text = if operation.contains("unicode") {
        "alpha café e\u{301}lan 東京 👩\u{200d}💻 omega".repeat((bytes / 48).max(1))
    } else if lines > 0 {
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
    if operation == "history-search-large" {
        let entries = (0..1_000).map(|i| format!("history item {i:04} alpha"));
        e.set_history_source(Some(
            HistorySearchSource::from_entries(
                entries,
                HistorySearchLimits::new(1_000, 64 * 1024, 128).unwrap(),
            )
            .unwrap(),
        ))
        .unwrap();
    }
    if matches!(
        operation,
        "word-right" | "word-delete-forward" | "word-right-unicode" | "word-delete-forward-unicode"
    ) {
        e.editor.home();
    }
    if matches!(operation, "undo-local" | "redo-local") {
        e.editor.home();
        e.editor.end();
        e.editor.insert("z").unwrap();
        if operation == "redo-local" {
            assert!(e.editor.undo());
        }
    }
    if operation == "undo-replace" {
        e.editor.replace(0..1, "z").unwrap();
    }
    if operation == "yank" {
        e.editor.kill_word_backward().unwrap();
    }
    e.set_submission_policy(SubmissionPolicy::Validated)
        .unwrap();
    e.start(Prompt::new("q2").unwrap(), (80, 24)).unwrap();
    if operation == "history-search-query" {
        e.apply(Input::Edit(EditCommand::HistorySearchOlder))
            .unwrap();
    }
    let candidate_count=if operation.contains("4096") {4096} else {10};
    let candidates = || vec![CompletionCandidate::new(0..text.len(), "replacement", "display").unwrap(); candidate_count];
    if matches!(operation, "menu-next" | "menu-accept" | "menu-next-4096" | "menu-page-4096" | "menu-first-4096" | "menu-last-4096" | "menu-resize-4096") {
        e.present_completions(CompletionSet::new(e.editor.revision(), candidates()).unwrap())
            .unwrap();
    }
    let item_count = if operation.ends_with("4096") { 4096 } else if operation.ends_with("1000") || operation == "path-large" || operation == "suggestion-history-large" { 1000 } else { 100 };
    let items=(0..item_count).map(|i|CompletionItem::new(&format!("alpha-{i:04}")).unwrap()).collect();
    let mut keymap=KeyMap::new();
    if operation=="key-custom" { keymap.bind(Key::meta(b'r').unwrap(),Action::Edit(EditAction::Redo)).unwrap(); }
    if operation=="key-maximum" { for byte in 0..32 { keymap.bind(Key::Control(byte),Action::Redraw).unwrap(); } for byte in 33..=126 { keymap.bind(Key::Meta(byte),Action::Redraw).unwrap(); } keymap.bind(Key::Meta(8),Action::Redraw).unwrap(); keymap.bind(Key::Meta(127),Action::Redraw).unwrap(); }
    let suggestion=if operation.starts_with("suggestion-") { Some(Suggestion::new(e.editor.revision()," suffix").unwrap()) } else {None};
    if matches!(operation,"suggestion-accept"|"suggestion-dismiss") { e.present_suggestion(suggestion.clone().unwrap()).unwrap(); }
    let path = operation.starts_with("path-").then(|| path_fixture(item_count));
    if matches!(operation, "validation" | "incomplete") {
        e.apply(Input::Request(Request::Submit)).unwrap();
    }
    let revision = e.editor.revision();
    State {
        e,
        intermediate: None,
        keymap,
        items,
        suggestion,
        path,
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
        "menu-next-4096" => e.completion_action(CompletionAction::Next).unwrap().1,
        "menu-page-4096" => e.completion_action(CompletionAction::PageNext).unwrap().1,
        "menu-first-4096" => e.completion_action(CompletionAction::First).unwrap().1,
        "menu-last-4096" => e.completion_action(CompletionAction::Last).unwrap().1,
        "menu-resize-4096" => e.apply(Input::Resize(40,12)).unwrap(),
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
        "word-left" | "word-left-unicode" => {
            e.editor.word_left();
            Effects::default()
        }
        "word-right" | "word-right-unicode" => {
            e.editor.word_right();
            Effects::default()
        }
        "word-delete-backward" | "word-delete-backward-unicode" => {
            e.editor.delete_word_backward();
            Effects::default()
        }
        "word-delete-forward" | "word-delete-forward-unicode" => {
            e.editor.delete_word_forward();
            Effects::default()
        }
        "undo-local" | "undo-replace" => {
            assert!(e.editor.undo());
            Effects::default()
        }
        "redo-local" => {
            assert!(e.editor.redo());
            Effects::default()
        }
        "history-search-small" | "history-search-large" => e
            .apply(Input::Edit(EditCommand::HistorySearchOlder))
            .unwrap(),
        "history-search-query" => e.apply(Input::Text("alpha".into())).unwrap(),
        "kill" => {
            e.editor.kill_word_backward().unwrap();
            Effects::default()
        }
        "yank" => {
            e.editor.yank().unwrap();
            Effects::default()
        }
        "key-default" => { black_box(s.keymap.get(Key::Named(NamedKey::Left))); Effects::default() }
        "key-custom" => { black_box(s.keymap.get(Key::Meta(b'r'))); Effects::default() }
        "key-maximum" => { black_box(s.keymap.get(Key::Meta(127))); Effects::default() }
        "key-bind" => { s.keymap.bind(Key::meta(b'z').unwrap(),Action::Edit(EditAction::Redo)).unwrap(); Effects::default() }
        "prefix-100"|"prefix-1000"|"prefix-4096" => { black_box(complete_prefix(&e.editor.analysis_snapshot(),0..0,"alpha",&s.items,MatchCase::Sensitive).unwrap()); Effects::default() }
        "fuzzy-100"|"fuzzy-1000" => { black_box(complete_fuzzy(&e.editor.analysis_snapshot(),0..0,"apa",&s.items,MatchCase::Sensitive).unwrap()); Effects::default() }
        "common-prefix-unicode" => { black_box(common_grapheme_prefix(&["界面e\u{301}x","界面e\u{301}y"])); Effects::default() }
        "path-small"|"path-large" => { let options=PathCompletionOptions::new(s.path.as_ref().unwrap()); black_box(complete_path(&e.editor.analysis_snapshot(),0..0,"alpha",&options).unwrap()); Effects::default() }
        "suggestion-present" => e.present_suggestion(s.suggestion.take().unwrap()).unwrap().1,
        "suggestion-stale" => { let suggestion=s.suggestion.take().unwrap(); e.editor.left(); let (outcome,effects)=e.present_suggestion(suggestion).unwrap(); assert_eq!(outcome,AnalysisOutcome::Stale); effects }
        "suggestion-accept" => e.suggestion_action(true).unwrap().1,
        "suggestion-dismiss" => e.suggestion_action(false).unwrap().1,
        "suggestion-history-small"|"suggestion-history-large" => { let values=s.items.iter().map(|i|i.value()).collect::<Vec<_>>(); black_box(suggest_from_history(&e.editor.analysis_snapshot(),&values).unwrap()); Effects::default() }
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
        "word-left",
        "word-right",
        "word-delete-backward",
        "word-delete-forward",
        "undo-local",
        "redo-local",
        "undo-replace",
        "history-search-small",
        "history-search-large",
        "history-search-query",
        "kill",
        "yank",
        "word-left-unicode",
        "word-right-unicode",
        "word-delete-backward-unicode",
        "word-delete-forward-unicode",
        "key-default", "key-custom", "key-maximum", "key-bind",
        "prefix-100", "prefix-1000", "prefix-4096", "fuzzy-100", "fuzzy-1000", "common-prefix-unicode",
        "path-small", "path-large", "menu-next-4096", "menu-page-4096", "menu-first-4096", "menu-last-4096", "menu-resize-4096",
        "suggestion-present", "suggestion-stale", "suggestion-accept", "suggestion-dismiss", "suggestion-history-small", "suggestion-history-large",
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
