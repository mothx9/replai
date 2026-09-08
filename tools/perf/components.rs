//! Private-source characterization binary, deliberately outside the production workspace.
#![allow(dead_code)]
#[path = "../../src/actions.rs"]
mod actions;
#[path = "../../src/capabilities.rs"]
mod capabilities;
#[path = "../../src/core.rs"]
mod core;
#[path = "../../src/engine.rs"]
mod engine;
#[path = "../../src/event.rs"]
mod event;
#[path = "../../src/input.rs"]
mod input;
#[path = "../../src/interaction.rs"]
mod interaction;
#[path = "../../src/keymap.rs"]
mod keymap;
#[path = "../../src/presentation.rs"]
mod presentation;
#[path = "../../src/protocol.rs"]
mod protocol;
#[cfg(all(not(test), any(target_os = "linux", target_os = "macos")))]
#[path = "../../tests/support/posix_pty.rs"]
mod pty_support;
#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
use terminal::pty_support;
#[path = "../../src/render.rs"]
mod render;
#[path = "../../src/substrate.rs"]
mod substrate;
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[path = "../../src/system.rs"]
mod system;
#[path = "../../src/terminal.rs"]
mod terminal;
pub use core::{EditError, Editor};
pub use event::{Error, Event};
pub use interaction::Interaction;
pub use presentation::{Foreground, Prompt, Role, Style, Theme};
mod allocation;
#[path = "../../src/document.rs"]
mod document;
mod documents;
use actions::{Input, Request};
pub use document::{Document, Text};
use engine::Engine;
use input::{Decoder, Key};
use presentation::Frame;
use render::{Mutation, Renderer};
use serde_json::{Value, json};
use std::{hint::black_box, time::Instant};
use unicode_segmentation::UnicodeSegmentation;

const LIMIT: usize = 1_048_576; // Explicit bounded host profile; public Editor has no default/max.
const CLASSES: &[(&str, &str)] = &[
    ("ascii", "abcdefgh "),
    ("latin", "café déjà "),
    ("cjk", "日本語中文 "),
    ("combining", "e\u{301}a\u{308} "),
    ("emoji", "🌍🚀 "),
    ("zwj", "👩‍💻👨‍👩‍👧‍👦 "),
    ("mixed", "fn main() { let x = {\"café\":42}; } prose 界 "),
    (
        "multiline",
        "let x = 42;\n{\"café\":\"界🌍\"}\nprose\tend\n",
    ),
];
fn text_at(size: usize, unit: &str) -> String {
    let mut s = unit.repeat(size / unit.len());
    // Exact byte sizes without splitting UTF-8 or a grapheme; remainder is ASCII.
    s.push_str(&"x".repeat(size - s.len()));
    s
}
fn editor(text: &str, pos: usize) -> Editor {
    let mut e = Editor::new(LIMIT, 100);
    e.insert(text).unwrap();
    e.replace(pos..pos, "").unwrap();
    e
}
fn mid(text: &str) -> usize {
    text.grapheme_indices(true)
        .map(|(i, _)| i)
        .find(|i| *i >= text.len() / 2)
        .unwrap_or(text.len())
}
fn theme() -> Theme {
    Theme::new(true, true, Some("xterm-256color"))
}
fn prompt() -> Prompt {
    Prompt::new("p").unwrap()
}
struct Harness {
    samples: usize,
    groups: usize,
    warmup: usize,
    filter: String,
}
impl Harness {
    fn measure<S, R>(
        &self,
        mut spec: Value,
        mut setup: impl FnMut() -> S,
        mut run: impl FnMut(&mut S) -> R,
        mut verify: impl FnMut(&S, &R) -> Value,
    ) {
        if !spec["id"].as_str().unwrap().contains(&self.filter) {
            return;
        }
        let mut times = Vec::with_capacity(self.samples * self.groups);
        let mut memories = Vec::with_capacity(self.samples * self.groups);
        let mut counters = Value::Null;
        for _ in 0..self.groups {
            for _ in 0..self.warmup {
                let mut state = setup();
                let output = run(black_box(&mut state));
                verify(&state, &output);
            }
            for _ in 0..self.samples {
                let mut state = setup();
                let a = allocation::start();
                let start = Instant::now();
                let output = black_box(run(black_box(&mut state)));
                let elapsed = start.elapsed().as_secs_f64() * 1e6;
                let memory = allocation::end(a);
                counters = verify(&state, &output);
                times.push(elapsed);
                memories.push(memory);
            }
        }
        spec["schema_version"] = json!(1);
        spec["status"] = json!("measured");
        spec["samples_per_group"] = json!(self.samples);
        spec["groups"] = json!(self.groups);
        spec["iterations"] = json!(times.len());
        spec["warmup_per_group"] = json!(self.warmup);
        spec["counters"] = counters;
        spec["mode"] = json!(if cfg!(feature = "allocations") {
            "allocation"
        } else {
            "latency"
        });
        spec["samples_us"] = if cfg!(feature = "allocations") {
            Value::Null
        } else {
            json!(times)
        };
        spec["allocation_samples"] = if cfg!(feature = "allocations") {
            json!(memories)
        } else {
            Value::Null
        };
        println!("{spec}");
    }
}
fn spec(component: &str, operation: &str, size: usize, class: &str, width: usize) -> Value {
    json!({"id":format!("{component}/{operation}/{class}/{size}/{width}"),
        "component":component,"operation":operation,"input_bytes":size,"text_class":class,
        "columns":width,"rows":24,"max_draft_bytes":LIMIT,"history_capacity":100})
}
fn mutation_counts(out: &[Mutation]) -> Value {
    json!({"logical_mutations":out.len(),"vt_bytes":protocol::encode(out, theme()).len(),
        "clear_lines":out.iter().filter(|m| matches!(m,Mutation::ClearLine)).count(),
        "logical_output_operations":1,"transport_writes":Value::Null,"syscalls":Value::Null,
        "append_transitions":usize::from(matches!(out,[Mutation::Text(_)])),
        "erase_transitions":usize::from(out.iter().any(|m|matches!(m,Mutation::ClearLine)))})
}
// Supplement: move over the Unicode grapheme itself, not corpus padding.
fn directed_movement(h: &Harness, sizes: &[usize]) {
    for &(class, unit) in CLASSES {
        for &size in sizes {
            let text = text_at(size, unit);
            let graphemes: Vec<_> = text
                .grapheme_indices(true)
                .filter(|(_, g)| class == "ascii" || !g.is_ascii())
                .collect();
            if graphemes.is_empty() {
                continue;
            }
            for (place, target) in [
                ("near_end", *graphemes.last().unwrap()),
                (
                    "middle",
                    *graphemes
                        .iter()
                        .find(|(i, _)| *i >= size / 2)
                        .unwrap_or_else(|| graphemes.last().unwrap()),
                ),
            ] {
                let (offset, g) = target;
                for direction in ["left", "right"] {
                    let (from, to) = if direction == "left" {
                        (offset + g.len(), offset)
                    } else {
                        (offset, offset + g.len())
                    };
                    h.measure(spec("movement",&format!("{direction}_{place}"),size,class,0),
                        ||editor(&text,from),|e|if direction=="left"{e.left()}else{e.right()},
                        |e,_|{assert_eq!(e.text(),text);assert_eq!(e.cursor(),to);
                            json!({"cursor_before":from,"cursor_after":to,"traversed_grapheme_bytes":g.len(),"traversed_non_ascii":!g.is_ascii()})});
                }
            }
        }
    }
    // A bounded but deliberately extreme single grapheme distinguishes document
    // length from grapheme complexity. It does not increase Editor's byte limit.
    for &size in sizes.iter().filter(|n| **n >= 64) {
        let marks = (size - 1) / 2;
        let mut text = "e".to_string();
        text.push_str(&"\u{301}".repeat(marks));
        let end = text.len();
        text.push_str(&"x".repeat(size - end));
        assert_eq!(text.len(), size);
        for direction in ["left", "right"] {
            let (from, to) = if direction == "left" {
                (end, 0)
            } else {
                (0, end)
            };
            h.measure(
                spec("movement", direction, size, "combining_chain", 0),
                || editor(&text, from),
                |e| {
                    if direction == "left" {
                        e.left()
                    } else {
                        e.right()
                    }
                },
                |e, _| {
                    assert_eq!(e.text(), text);
                    assert_eq!(e.cursor(), to);
                    json!({"traversed_grapheme_bytes":end,"combining_marks":marks})
                },
            );
        }
        let e = editor(&text, end);
        let p = prompt();
        h.measure(spec("movement","chain_layout",size,"combining_chain",80),||(),|_|Frame::new(&e,&p,80,24),
            |_,f|{assert_eq!(f.cursor.row,0);assert_eq!(f.cursor.col,4);json!({"visible_logical_rows":f.lines.len(),"draft_graphemes":text.graphemes(true).count()})});
    }
}

fn render_edges(h: &Harness) {
    let p = prompt();
    for size in [75, 76, 1836] {
        let old = editor(&"a".repeat(size), size);
        let mut new = editor(old.text(), size);
        new.insert("X").unwrap();
        h.measure(
            spec("render_edge", "append_wrap", size, "ascii", 80),
            || {
                let mut r = Renderer::default();
                r.redraw(&old, &p, (80, 24));
                (r, Some(Frame::new(&new, &p, 80, 24)))
            },
            |(r, f)| r.transition(f.take().unwrap()),
            |_, out| {
                if size == 75 {
                    assert_eq!(out, &vec![Mutation::Text("X".into())]);
                } else {
                    assert!(out.iter().any(|m| matches!(m, Mutation::ClearLine)));
                }
                mutation_counts(out)
            },
        );
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let option = |key: &str, default: &str| {
        args.windows(2)
            .find(|v| v[0] == key)
            .map(|v| v[1].clone())
            .unwrap_or(default.into())
    };
    if let Some(pair) = args.windows(2).find(|v| v[0] == "--predict") {
        let c: Value = serde_json::from_slice(&std::fs::read(&pair[1]).unwrap()).unwrap();
        let initial = c["initial"].as_str().unwrap_or("");
        let mut e = Engine::new(editor(
            initial,
            c["cursor"].as_u64().map_or(initial.len(), |n| n as usize),
        ));
        e.editor.admit_history("history first").unwrap();
        e.editor.admit_history("history second").unwrap();
        let initial_effect = e.start(prompt(), (80, 24)).unwrap();
        let initial_bytes = protocol::encode(&initial_effect.mutations, theme());
        let mut out = String::new();
        let mut mutations = 0;
        if let Some(width) = c["resize"].as_u64() {
            let fx = e.apply(Input::Resize(width as usize, 24)).unwrap();
            mutations += fx.mutations.len();
            out.push_str(&protocol::encode(&fx.mutations, theme()));
        }
        let mut d = Decoder::new(LIMIT);
        for b in c["input"].as_str().unwrap_or("").as_bytes() {
            if let Some(k) = d.feed(*b) {
                let fx = e.apply_deferred(keymap::binding(k).unwrap()).unwrap();
                mutations += fx.mutations.len();
                out.push_str(&protocol::encode(&fx.mutations, theme()));
                if fx.event == Some(Event::CompletionRequested) {
                    let fx = e
                        .complete(0..e.editor.text().len(), c["replacement"].as_str().unwrap())
                        .unwrap();
                    mutations += fx.mutations.len();
                    out.push_str(&protocol::encode(&fx.mutations, theme()));
                }
            }
        }
        let fx = e.flush();
        mutations += fx.mutations.len();
        out.push_str(&protocol::encode(&fx.mutations, theme()));
        println!(
            "{}",
            json!({"output":out,"initial_output":initial_bytes,"text":e.editor.text(),"cursor":e.editor.cursor(),"logical_mutations":mutations})
        );
        return;
    }
    let smoke = args.iter().any(|s| s == "--smoke");
    let h = Harness {
        samples: option("--samples", if smoke { "3" } else { "21" })
            .parse()
            .unwrap(),
        groups: option("--groups", if smoke { "1" } else { "3" })
            .parse()
            .unwrap(),
        warmup: 2,
        filter: option("--filter", ""),
    };
    assert!(h.samples > 0 && h.groups > 0);
    let sizes: &[usize] = if smoke {
        &[0, 64, 1024]
    } else {
        &[0, 8, 64, 1024, 4096, 16384, 65536, 262144, 1048576]
    };
    directed_movement(
        &h,
        if smoke {
            &[64]
        } else {
            &[64, 4096, 65536, 1048576]
        },
    );
    render_edges(&h);
    documents::measure(&h);
    interaction_scaling(&h, smoke);
    h.measure(
        spec("control", "timer_black_box", 0, "none", 0),
        || 0usize,
        |s| black_box(*s),
        |_, r| {
            assert_eq!(*r, 0);
            json!({})
        },
    );
    // Whole decoder sequence, emission count and semantic correctness outside timing.
    for size in if smoke {
        vec![64]
    } else {
        vec![64, 1024, 16384, 65536]
    } {
        let mut streams: Vec<(String, Vec<u8>)> = CLASSES[..6]
            .iter()
            .map(|(c, u)| (c.to_string(), text_at(size, u).into_bytes()))
            .collect();
        streams.extend([
            (
                "arrows".into(),
                b"\x1b[A\x1b[B\x1b[C\x1b[D".repeat(size / 12 + 1),
            ),
            (
                "home_end_delete".into(),
                b"\x1b[H\x1b[F\x1b[3~".repeat(size / 10 + 1),
            ),
            (
                "mixed_valid".into(),
                "aé界\x1b[D\x1b[C".as_bytes().repeat(size / 12 + 1),
            ),
            ("invalid_utf8".into(), vec![0xff; size]),
            ("unknown_escape".into(), b"\x1b[999~".repeat(size / 6 + 1)),
            (
                "malformed_bounded".into(),
                [b"\x1b[".as_slice(), &vec![b'1'; size], b"~a"].concat(),
            ),
        ]);
        for (class, bytes) in streams {
            let mut d = Decoder::new(LIMIT);
            let expected: Vec<_> = bytes.iter().filter_map(|b| d.feed(*b)).collect();
            assert!(!d.pending());
            if class == "malformed_bounded" {
                assert_eq!(
                    expected,
                    vec![
                        Key::Rejected(EditError::InvalidSequence),
                        Key::Text("a".into())
                    ]
                );
            }
            let expected_count = expected.len();
            h.measure(
                spec("decoder", "stream", bytes.len(), &class, 0),
                || Decoder::new(LIMIT),
                |d| {
                    let mut n = 0;
                    for b in &bytes {
                        if let Some(k) = d.feed(black_box(*b)) {
                            black_box(k);
                            n += 1;
                        }
                    }
                    n
                },
                |d, n| {
                    assert!(!d.pending());
                    assert_eq!(*n, expected_count);
                    json!({"logical_inputs":n})
                },
            );
        }
    }
    for (name, make) in [
        ("navigation", (|| Key::Left) as fn() -> Key),
        ("request", || Key::Tab),
        ("text", || Key::Text("é".into())),
        ("rejected", || Key::Rejected(EditError::Capacity)),
    ] {
        h.measure(
            spec("keymap", name, 0, "normalized", 0),
            || Some(make()),
            |s| keymap::binding(s.take().unwrap()).unwrap(),
            |_, action| {
                assert!(match name {
                    "navigation" => matches!(action, Input::Edit(actions::EditCommand::Left)),
                    "request" => matches!(action, Input::Request(Request::Completion)),
                    "text" => matches!(action,Input::Text(s) if s=="é"),
                    _ => matches!(action, Input::Rejected(EditError::Capacity)),
                });
                json!({})
            },
        );
    }
    for &(class, unit) in CLASSES {
        for &size in sizes {
            let text = text_at(size, unit);
            let middle = mid(&text);
            for op in [
                "append",
                "insert_begin",
                "insert_middle",
                "backspace_end",
                "backspace_middle",
                "delete_end",
                "delete_middle",
                "left",
                "right",
                "home",
                "end",
                "completion_short",
                "completion_large",
                "clear",
                "history_admit",
                "history_up",
                "history_down",
                "draft_restore",
            ] {
                let pos = match op {
                    "insert_begin" | "end" => 0,
                    "insert_middle" | "backspace_middle" | "delete_middle" | "right"
                    | "completion_short" | "completion_large" => middle,
                    _ => size,
                };
                let replacement = if op == "completion_large" {
                    "R".repeat(4096.min(size.saturating_sub(middle).max(1)))
                } else {
                    "R".into()
                };
                let mut expected = text.clone();
                let mut cursor = pos;
                match op {
                    "append" | "insert_begin" | "insert_middle" if size < LIMIT => {
                        expected.insert(pos, 'X');
                        cursor = pos + 1;
                    }
                    "backspace_end" | "backspace_middle" => {
                        let start = text[..pos]
                            .grapheme_indices(true)
                            .next_back()
                            .map_or(0, |(i, _)| i);
                        expected.replace_range(start..pos, "");
                        cursor = start;
                    }
                    "delete_end" | "delete_middle" => {
                        let end = pos + text[pos..].graphemes(true).next().map_or(0, str::len);
                        expected.replace_range(pos..end, "");
                    }
                    "left" => {
                        cursor = text[..pos]
                            .grapheme_indices(true)
                            .next_back()
                            .map_or(0, |(i, _)| i)
                    }
                    "right" => {
                        cursor = pos + text[pos..].graphemes(true).next().map_or(0, str::len)
                    }
                    "home" => cursor = 0,
                    "end" => cursor = size,
                    "completion_short" | "completion_large" => {
                        expected.replace_range(pos..size, &replacement);
                        cursor = pos + replacement.len();
                    }
                    "clear" => {
                        expected.clear();
                        cursor = 0;
                    }
                    "history_up" => {
                        expected = "recalled".into();
                        cursor = 8;
                    }
                    "history_down" => {
                        expected = "newest".into();
                        cursor = 6;
                    }
                    _ => {}
                }
                let rejected = expected.len() > LIMIT
                    || (size == LIMIT && matches!(op, "append" | "insert_begin" | "insert_middle"));
                if rejected {
                    expected = text.clone();
                    cursor = pos;
                }
                h.measure(spec("editor",op,size,class,0), || {
                    let mut e=editor(&text,pos);
                    if matches!(op,"history_up"|"history_down"|"draft_restore") {e.admit_history("recalled").unwrap();}
                    if op=="history_down" {e.admit_history("newest").unwrap();e.history_up();e.history_up();}
                    if op=="draft_restore" {e.history_up();}
                    e
                }, |e| match op {
                    "append"|"insert_begin"|"insert_middle"=>e.insert("X"),
                    "completion_short"|"completion_large"=>e.replace(pos..size,&replacement),
                    "history_admit"=>e.admit_history(&text),
                    _=>{match op {"backspace_end"|"backspace_middle"=>e.backspace(),"delete_end"|"delete_middle"=>e.delete(),
                        "left"=>e.left(),"right"=>e.right(),"home"=>e.home(),"end"=>e.end(),"clear"=>e.clear(),
                        "history_up"=>e.history_up(),"history_down"|"draft_restore"=>e.history_down(),_=>unreachable!()} Ok(())}
                }, |e,r| {assert_eq!(r.is_err(),rejected,"{op}/{size}/{class}");assert_eq!(e.text(),expected,"{op}/{size}/{class}");assert_eq!(e.cursor(),cursor,"{op}/{size}/{class}");json!({"rejected_capacity":rejected,"result_bytes":e.text().len(),"cursor":e.cursor()})});
            }
            for width in if smoke { vec![80] } else { vec![20, 80, 240] } {
                for (position, pos) in [("begin", 0), ("middle", middle), ("end", size)] {
                    let e = editor(&text, pos);
                    let p = prompt();
                    h.measure(spec("layout",position,size,class,width), ||(),
                        |_|Frame::new(black_box(&e),black_box(&p),width,24), |_,f| {
                        assert!(!f.lines.is_empty() && f.lines.len()<=23);assert!(f.cursor.row<f.lines.len());assert!(f.cursor.col<width);
                        json!({"visible_logical_rows":f.lines.len(),"cursor_row":f.cursor.row,"cursor_col":f.cursor.col})});
                }
            }
        }
    }
    // Prompt newline is outside the current contract. Wrapped prompts are admitted.
    assert_eq!(
        Prompt::new("first\nsecond").unwrap_err(),
        EditError::InvalidText
    );
    for width in [20, 80, 240] {
        let p = Prompt::new(&"prompt ".repeat(80)).unwrap();
        let e = editor("draft", 5);
        h.measure(
            spec("layout", "wrapped_prompt", 560, "ascii", width),
            || (),
            |_| Frame::new(&e, &p, width, 24),
            |_, f| {
                assert!(f.cursor.row < 23);
                json!({"visible_logical_rows":f.lines.len()})
            },
        );
    }
    // Pre-built frame seam: layout excluded. Every transition is checked against
    // fresh geometry, with VT byte/mutation counts computed outside the timer.
    for size in if smoke {
        vec![8, 1024]
    } else {
        vec![8, 64, 1024, 4096, 16384, 65536, 262144, 1048576]
    } {
        for (class, unit) in [CLASSES[0], CLASSES[5], CLASSES[7]] {
            let text = text_at(size, unit);
            for op in [
                "append",
                "cursor_left",
                "middle_insert",
                "middle_delete",
                "history_replace",
                "resize",
                "multiline_change",
                "full_redraw",
                "external_output",
            ] {
                if size == LIMIT && matches!(op, "append" | "middle_insert") {
                    if format!("render/{op}").contains(&h.filter) || h.filter.is_empty() {
                        let mut row = spec("render", op, size, class, 80);
                        row["schema_version"] = json!(1);
                        row["status"] = json!("unsupported");
                        row["mode"] = json!(if cfg!(feature = "allocations") {
                            "allocation"
                        } else {
                            "latency"
                        });
                        row["reason"] = json!(
                            "configured 1 MiB draft is full; no successful insertion transition exists"
                        );
                        println!("{row}");
                    }
                    continue;
                }
                let mut old = editor(&text, text.len());
                let mut new = editor(&text, text.len());
                let p = prompt();
                let mut width = 80;
                match op {
                    "append" => {
                        if size < LIMIT {
                            new.insert("X").unwrap();
                        }
                    }
                    "cursor_left" => new.left(),
                    "middle_insert" => {
                        old.replace(mid(&text)..mid(&text), "").unwrap();
                        new.replace(mid(&text)..mid(&text), if size < LIMIT { "X" } else { "" })
                            .unwrap();
                    }
                    "middle_delete" => {
                        old.replace(mid(&text)..mid(&text), "").unwrap();
                        new.replace(mid(&text)..mid(&text), "").unwrap();
                        new.delete();
                    }
                    "history_replace" => {
                        new.clear();
                        new.insert("recalled command").unwrap();
                    }
                    "resize" => width = 20,
                    "multiline_change" => {
                        new.replace(mid(&text)..text.len(), "first\nsecond")
                            .unwrap();
                    }
                    _ => {}
                }
                let setup = || {
                    let mut r = Renderer::default();
                    r.redraw(&old, &p, (80, 24));
                    (r, Some(Frame::new(&new, &p, width, 24)))
                };
                h.measure(
                    spec("render", op, size, class, width),
                    setup,
                    |(r, f)| {
                        let mut out = if matches!(op, "full_redraw" | "external_output") {
                            r.erase()
                        } else {
                            vec![]
                        };
                        out.extend(r.transition(f.take().unwrap()));
                        out
                    },
                    |_, out| {
                        let expected = Frame::new(&new, &p, width, 24);
                        assert!(expected.cursor.row < 23);
                        mutation_counts(out)
                    },
                );
                let (mut r, mut f) = setup();
                let mut mutations = if matches!(op, "full_redraw" | "external_output") {
                    r.erase()
                } else {
                    vec![]
                };
                mutations.extend(r.transition(f.take().unwrap()));
                // Independent VT terminal-state equivalence for stable dimensions
                // and single-cell ASCII; Unicode geometry is asserted by core oracle.
                if class == "ascii" && width == 80 {
                    let mut incremental = vt100::Parser::new(24, 80, 0);
                    incremental.process(
                        protocol::encode(&Frame::new(&old, &p, 80, 24).draw(), theme()).as_bytes(),
                    );
                    incremental.process(protocol::encode(&mutations, theme()).as_bytes());
                    let mut reference = vt100::Parser::new(24, 80, 0);
                    reference.process(
                        protocol::encode(&Frame::new(&new, &p, 80, 24).draw(), theme()).as_bytes(),
                    );
                    assert_eq!(
                        incremental.screen().contents(),
                        reference.screen().contents(),
                        "render {op}/{size}"
                    );
                    assert_eq!(
                        incremental.screen().cursor_position(),
                        reference.screen().cursor_position()
                    );
                }
                h.measure(
                    spec("vt_encode", op, size, class, width),
                    || (),
                    |_| protocol::encode(black_box(&mutations), theme()),
                    |_, s| {
                        assert_eq!(s, &protocol::encode(&mutations, theme()));
                        json!({"vt_bytes":s.len(),"logical_mutations":mutations.len()})
                    },
                );
            }
        }
    }
    // Paste normalization (CRLF and control validation) belongs to Decoder::feed
    // on the closing delimiter. The keymap only moves the normalized String.
    for size in if smoke {
        vec![1024]
    } else {
        vec![1024, 4096, 16384, 65536, 262144, 1048576]
    } {
        for (class, unit) in [
            ("ascii", "abcd "),
            ("utf8_prose", "café 界 prose "),
            ("source", "fn main() {\n    x();\n}\n"),
            ("json", "{\"key\":42,\"str\":\"café\"}\n"),
            ("mixed_multiline", "café\r\n界\r\nfn() {}\n"),
        ] {
            let text = text_at(size, unit);
            let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
            let framed = [b"\x1b[200~".as_slice(), text.as_bytes(), b"\x1b[201~"].concat();
            h.measure(
                spec("paste_decoder", "framing_normalization", size, class, 0),
                || Decoder::new(LIMIT),
                |d| {
                    let mut result = None;
                    for b in &framed {
                        if let Some(k) = d.feed(*b) {
                            assert!(result.is_none());
                            result = Some(k);
                        }
                    }
                    result
                },
                |d, r| {
                    assert!(!d.pending());
                    assert_eq!(r, &Some(Key::Text(normalized.clone())));
                    json!({"logical_inputs":1,"wire_input_bytes":framed.len()})
                },
            );
            h.measure(
                spec("paste_normalization", "final_delimiter", size, class, 0),
                || {
                    let mut d = Decoder::new(LIMIT);
                    for b in &framed[..framed.len() - 1] {
                        assert!(d.feed(*b).is_none());
                    }
                    d
                },
                |d| d.feed(*framed.last().unwrap()),
                |_, r| {
                    assert_eq!(r, &Some(Key::Text(normalized.clone())));
                    json!({"normalized_bytes":normalized.len()})
                },
            );
            h.measure(
                spec("paste_keymap", "normalized_text", size, class, 0),
                || Some(Key::Text(normalized.clone())),
                |s| keymap::binding(s.take().unwrap()).unwrap(),
                |_, r| {
                    assert_eq!(r, &Input::Text(normalized.clone()));
                    json!({})
                },
            );
            h.measure(
                spec("paste_editor", "insert", size, class, 0),
                || Editor::new(LIMIT, 100),
                |e| e.insert(&normalized),
                |e, r| {
                    assert!(r.is_ok());
                    assert_eq!(e.text(), normalized);
                    assert_eq!(e.cursor(), normalized.len());
                    json!({})
                },
            );
            let e = editor(&normalized, normalized.len());
            let p = prompt();
            h.measure(
                spec("paste_layout_render", "redraw", size, class, 80),
                Renderer::default,
                |r| r.redraw(&e, &p, (80, 24)),
                |_, r| mutation_counts(r),
            );
        }
    }
    for limit in [0, 65536, LIMIT] {
        let data = [
            b"\x1b[200~".as_slice(),
            &vec![b'x'; limit + 1],
            b"\x1b[201~",
        ]
        .concat();
        let mut e = Engine::new(editor("draft", 5));
        e.start(prompt(), (80, 24)).unwrap();
        let mut d = Decoder::new(limit);
        let mut events = vec![];
        for b in &data {
            if let Some(k) = d.feed(*b) {
                events.push(e.apply(keymap::binding(k).unwrap()).unwrap().event);
            }
        }
        assert_eq!(events, vec![Some(Event::Rejected(EditError::Capacity))]);
        assert_eq!(e.editor.text(), "draft");
    }
    for capacity in if smoke {
        vec![10]
    } else {
        vec![10, 100, 1000, 10000, 100000]
    } {
        let entry = "h".repeat(64);
        let history = || {
            let mut e = Editor::new(LIMIT, capacity);
            for _ in 0..capacity {
                e.admit_history(&entry).unwrap();
            }
            e.insert("unsent").unwrap();
            e.left();
            e
        };
        for op in ["admit_evict", "up", "down", "restore"] {
            let mut s = spec("history", op, 64, "ascii", 0);
            s["history_capacity"] = json!(capacity);
            s["id"] = json!(format!("{}/capacity-{capacity}", s["id"].as_str().unwrap()));
            h.measure(
                s,
                || {
                    let mut e = history();
                    if matches!(op, "down" | "restore") {
                        e.history_up();
                    }
                    if op == "down" {
                        e.history_up();
                    }
                    e
                },
                |e| match op {
                    "admit_evict" => e.admit_history("newest").unwrap(),
                    "up" => e.history_up(),
                    _ => e.history_down(),
                },
                |e, _| {
                    let expected = if op == "restore" || op == "admit_evict" {
                        "unsent"
                    } else {
                        &entry
                    };
                    assert_eq!(e.text(), expected);
                    if op == "restore" {
                        assert_eq!(e.cursor(), 5);
                    }
                    json!({"entries":capacity})
                },
            );
        }
        let mut s = spec("memory", "history_growth", 64, "ascii", 0);
        s["history_capacity"] = json!(capacity);
        s["id"] = json!(format!("{}/capacity-{capacity}", s["id"].as_str().unwrap()));
        h.measure(
            s,
            || (),
            |_| history(),
            |e, _| {
                let _ = e;
                json!({"entries":capacity,"entry_bytes":64})
            },
        );
        // Exact eviction oracle separate from timing.
        let mut e = Editor::new(64, capacity);
        e.admit_history("oldest").unwrap();
        for _ in 1..capacity {
            e.admit_history("keep").unwrap();
        }
        e.admit_history("newest").unwrap();
        for _ in 0..capacity {
            e.history_up();
        }
        assert_eq!(e.text(), "keep");
    }
    h.measure(spec("memory","base_interaction",0,"none",0),||(),|_|Interaction::new(Editor::new(LIMIT,100)),|_,i|{assert!(!i.is_open());json!({"editor_stack_bytes":std::mem::size_of::<Editor>(),"interaction_stack_bytes":std::mem::size_of::<Interaction>()})});
    for &size in sizes {
        let t = text_at(size, "abcd ");
        h.measure(
            spec("memory", "draft_growth", size, "ascii", 0),
            || (),
            |_| editor(&t, size),
            |_, e| {
                assert_eq!(e.text(), t);
                json!({"retained_text_bytes":e.text().len()})
            },
        );
    }
    for size in if smoke {
        vec![80, 1024]
    } else {
        vec![16, 80, 1024, 4096, 16384]
    } {
        let text = text_at(size, "host output line\n");
        let draft = "retained draft 界";
        h.measure(
            spec(
                "external_output",
                "engine_transaction",
                size,
                "multiline",
                80,
            ),
            || {
                let mut e = Engine::new(editor(draft, mid(draft)));
                e.start(prompt(), (80, 24)).unwrap();
                e
            },
            |e| e.external_output(Role::Dim, &text).unwrap(),
            |e, r| {
                assert_eq!(e.editor.text(), draft);
                assert_eq!(e.editor.cursor(), mid(draft));
                mutation_counts(&r.mutations)
            },
        );
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    for size in [1, 80, 1024, 4096] {
        use substrate::Transport;
        let bytes = vec![b'x'; size];
        // This family times enqueue without a concurrent reader. Probe with
        // nonblocking I/O first: a Darwin PTY can fill below 4 KiB. Waiting for
        // a reader that only runs in verification would deadlock the harness.
        let admitted = {
            let (_master, _slave, resource, _saved) = transport_fixture();
            let mut resource = resource.into_inner();
            let flags = rustix::fs::fcntl_getfl(&resource.output).unwrap();
            rustix::fs::fcntl_setfl(&resource.output, flags | rustix::fs::OFlags::NONBLOCK)
                .unwrap();
            let admitted = match resource.write(&bytes) {
                Ok(()) => true,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => false,
                Err(e) => panic!("transport enqueue probe failed: {e}"),
            };
            resource.restore().unwrap();
            admitted
        };
        if !admitted {
            let mut row = spec("transport", "kernel_enqueue", size, "ascii", 80);
            if row["id"].as_str().unwrap().contains(&h.filter) {
                row["schema_version"] = json!(1);
                row["status"] = json!("unsupported");
                row["mode"] = json!(if cfg!(feature = "allocations") {
                    "allocation"
                } else {
                    "latency"
                });
                row["reason"] = json!(
                    "PTY queue cannot hold this payload without a concurrent reader; the enqueue-only fixture does not measure backpressure"
                );
                println!("{row}");
            }
            continue;
        }
        h.measure(
            spec("transport", "kernel_enqueue", size, "ascii", 80),
            transport_fixture,
            |(_, _, resource, _)| resource.get_mut().write(&bytes),
            |(master, slave, resource, saved), result| {
                assert!(result.is_ok());
                let mut received = vec![0; size];
                let mut read = 0;
                while read < size {
                    read += rustix::io::read(master, &mut received[read..]).unwrap();
                }
                assert_eq!(received, bytes);
                resource.borrow_mut().restore().unwrap();
                assert_eq!(
                    &format!("{:?}", rustix::termios::tcgetattr(slave).unwrap()),
                    saved
                );
                json!({"transport_calls":1,"payload_bytes":size,"syscalls":Value::Null})
            },
        );
    }
    for op in [
        "completion_request",
        "completion_short",
        "completion_large",
        "append",
    ] {
        let replacement = if op == "completion_large" {
            "R".repeat(4096)
        } else {
            "replacement".into()
        };
        h.measure(
            spec("interaction", op, 64, "ascii", 80),
            || {
                let mut e = Engine::new(editor(&"x".repeat(64), 64));
                e.start(prompt(), (80, 24)).unwrap();
                e
            },
            |e| {
                let effects = match op {
                    "completion_request" => e.apply(Input::Request(Request::Completion)).unwrap(),
                    "append" => e.apply(Input::Text("X".into())).unwrap(),
                    _ => e.complete(0..64, &replacement).unwrap(),
                };
                let bytes = protocol::encode(&effects.mutations, theme());
                (effects, bytes)
            },
            |e, (effects, bytes)| {
                match op {
                    "completion_request" => {
                        assert_eq!(effects.event, Some(Event::CompletionRequested))
                    }
                    "append" => assert_eq!(e.editor.text(), format!("{}X", "x".repeat(64))),
                    _ => assert_eq!(e.editor.text(), replacement),
                }
                json!({"vt_bytes":bytes.len(),"logical_mutations":effects.mutations.len()})
            },
        );
    }
}

fn interaction_scaling(h: &Harness, smoke: bool) {
    use actions::EditCommand;
    for size in if smoke {
        vec![64, 4096]
    } else {
        vec![8, 64, 1024, 4096, 65536, 262144, LIMIT]
    } {
        for (class, unit) in [CLASSES[0], CLASSES[3], CLASSES[5], CLASSES[7]] {
            let text = text_at(size, unit);
            for op in [
                "append_end",
                "backspace_end",
                "cursor_left",
                "middle_insert",
                "completion_range",
            ] {
                if size == LIMIT && matches!(op, "append_end" | "middle_insert") {
                    continue;
                }
                let cursor = if matches!(op, "middle_insert" | "completion_range") {
                    mid(&text)
                } else {
                    text.len()
                };
                let mut expected = editor(&text, cursor);
                match op {
                    "append_end" | "middle_insert" => expected.insert("X").unwrap(),
                    "backspace_end" => expected.backspace(),
                    "cursor_left" => expected.left(),
                    "completion_range" => expected.replace(0..cursor, "selected").unwrap(),
                    _ => unreachable!(),
                }
                h.measure(spec("interaction_edit", op, size, class, 80),
                    || {
                        let mut e = Engine::new(editor(&text, cursor));
                        e.start(prompt(), (80,24)).unwrap();
                        e
                    },
                    |e| {
                        let effects = match op {
                            "append_end" | "middle_insert" => e.apply(Input::Text("X".into())).unwrap(),
                            "backspace_end" => e.apply(Input::Edit(EditCommand::Backspace)).unwrap(),
                            "cursor_left" => e.apply(Input::Edit(EditCommand::Left)).unwrap(),
                            "completion_range" => e.complete(0..cursor, "selected").unwrap(),
                            _ => unreachable!(),
                        };
                        let bytes = protocol::encode(&effects.mutations, theme());
                        (effects, bytes)
                    },
                    |e, (effects, bytes)| {
                        assert_eq!((e.editor.text(), e.editor.cursor()), (expected.text(), expected.cursor()));
                        json!({"vt_bytes":bytes.len(),"logical_mutations":effects.mutations.len(),"logical_transport_writes":usize::from(!bytes.is_empty()),"coverage":"added during MACOS/PERF; no original P0 timing pair"})
                    });
            }
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn transport_fixture() -> (
    std::os::fd::OwnedFd,
    std::os::fd::OwnedFd,
    std::cell::RefCell<system::Resource>,
    String,
) {
    let (master, slave) = pty_support::pair();
    rustix::termios::tcsetwinsize(
        &slave,
        rustix::termios::Winsize {
            ws_row: 24,
            ws_col: 80,
            ws_xpixel: 0,
            ws_ypixel: 0,
        },
    )
    .unwrap();
    let saved = format!("{:?}", rustix::termios::tcgetattr(&slave).unwrap());
    let (resource, _) = system::Resource::acquire(&slave, &slave).unwrap();
    (master, slave, std::cell::RefCell::new(resource), saved)
}
