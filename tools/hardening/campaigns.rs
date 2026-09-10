use crate::{
    actions::{EditCommand as E, Input, Request as R},
    engine::{Effects, Engine},
    input::{Decoder, Key},
    model::{LIMIT, Model, WORDS},
    presentation::Frame,
    render::Mutation,
    *,
};
use unicode_segmentation::UnicodeSegmentation;

fn check_editor(e: &Editor) {
    assert!(e.text().len() <= e.capacity());
    assert!(
        e.text()
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .chain([e.text().len()])
            .any(|i| i == e.cursor())
    );
}
fn effects(fx: &Effects) {
    for m in &fx.mutations {
        if let Mutation::Text(s) = m {
            assert!(!s.chars().any(char::is_control), "unsafe text run {s:?}");
        }
    }
    assert!(fx.mutations.len() < 100_000);
}
fn opened() -> Engine {
    let mut e = Engine::new(Editor::new(LIMIT, 8));
    e.set_submission_policy(SubmissionPolicy::Validated)
        .unwrap();
    e.start(Prompt::new("test").unwrap(), (40, 10)).unwrap();
    e
}

/// Decoder fragmentation equivalence, expiry, atomic paste and normalized action safety.
pub fn protocol_case(data: &[u8]) {
    let data = &data[..data.len().min(4096)];
    for chunk in [1, 7, 4096] {
        let mut d = Decoder::new(LIMIT);
        let mut e = opened();
        let mut keys = Vec::new();
        for part in data.chunks(chunk) {
            for &b in part {
                if let Some(k) = d.feed(b) {
                    keys.push(format!("{k:?}"));
                    if let Ok(input) = keymap::binding(k) {
                        if !e.is_open() {
                            e.start(Prompt::new("test").unwrap(), (40, 10)).unwrap();
                        }
                        if let Ok(fx) = e.apply_deferred(input) {
                            effects(&fx);
                        }
                        check_editor(&e.editor);
                    }
                }
            }
            effects(&e.flush());
        }
        let expired = d.expire();
        assert!(!d.pending());
        assert_eq!(d.expire(), None);
        let mut reference = Decoder::new(LIMIT);
        let reference_keys: Vec<_> = data
            .iter()
            .filter_map(|b| reference.feed(*b))
            .map(|k| format!("{k:?}"))
            .collect();
        assert_eq!(keys, reference_keys);
        assert_eq!(expired, reference.expire());
    }
    // Transport bytes inside paste must never become control actions. Escape
    // bytes in arbitrary payload may include a delimiter, so use safe UTF-8 here.
    let safe: String = String::from_utf8_lossy(data)
        .chars()
        .filter(|c| !c.is_control() || *c == '\t' || *c == '\n')
        .collect();
    let mut d = Decoder::new(safe.len() + 1);
    for b in b"\x1b[200~" {
        assert_eq!(d.feed(*b), None);
    }
    for b in safe.bytes() {
        assert_eq!(d.feed(b), None);
    }
    let mut received = Vec::new();
    for b in b"\x1b[201~" {
        if let Some(k) = d.feed(*b) {
            received.push(k);
        }
    }
    assert_eq!(received, vec![Key::Text(safe)]);
}

/// Full-string editor/history oracle, including rejected edits and cursor revisions.
pub fn editor_case(data: &[u8]) {
    let mut e = Editor::new(LIMIT, 8);
    let mut model = Model::default();
    for c in data.as_chunks::<4>().0.iter().take(128) {
        let text = WORDS[c[3] as usize % WORDS.len()];
        model.operation(&mut e, c[0], c[1], c[2], text);
    }
}

/// Revision-bound result/lifecycle composition. Snapshots are retained across edits.
pub fn results_case(data: &[u8]) {
    let mut e = opened();
    let mut snapshots = vec![e.editor.analysis_snapshot()];
    for c in data.as_chunks::<4>().0.iter().take(128) {
        if !e.is_open() {
            e.start(Prompt::new("test").unwrap(), (40, 10)).unwrap();
        }
        let before = e.editor.analysis_snapshot();
        let selection = e.completion_selection();
        let old_analysis = format!("{:?}", e.analysis_presentation());
        let old_diagnostics = format!("{:?}", e.diagnostics());
        let s = &snapshots[c[1] as usize % snapshots.len()];
        let stale = s.revision() != before.revision();
        let range = if c[2] & 1 == 0 {
            0..s.text().len()
        } else {
            c[2] as usize..c[3] as usize
        };
        let mut refusal = false;
        match c[0] % 24 {
            0 => {
                if snapshots.len() == 8 {
                    snapshots.remove(0);
                }
                snapshots.push(e.editor.analysis_snapshot());
            }
            1..=4 => {
                if let Ok(fx) = e.apply(Input::Text(WORDS[c[3] as usize % WORDS.len()].into())) {
                    effects(&fx);
                }
            }
            5 => {
                effects(&e.apply(Input::Edit(E::Left)).unwrap());
            }
            6 => {
                effects(&e.apply(Input::Edit(E::Right)).unwrap());
            }
            7 => {
                effects(&e.apply(Input::Edit(E::Backspace)).unwrap());
            }
            8 => {
                let Ok(candidate) = CompletionCandidate::new(range, "done", "label") else {
                    continue;
                };
                let candidate = candidate.with_annotation("host description").unwrap();
                let set =
                    CompletionSet::new(s.revision(), vec![candidate.clone(), candidate]).unwrap();
                if let Ok((outcome, fx)) = e.present_completions(set) {
                    effects(&fx);
                    refusal = stale;
                    assert_eq!(outcome == AnalysisOutcome::Stale, stale);
                    if stale {
                        assert!(fx.mutations.is_empty() && fx.event.is_none());
                    }
                } else {
                    assert!(!stale);
                }
            }
            9 => {
                let spans = AnalysisSpan::new(range, Role::Accent)
                    .map(|s| vec![s])
                    .unwrap_or_default();
                let result = AnalysisPresentation::new(
                    s.revision(),
                    spans,
                    Some(Hint::new("derived", Role::Dim).unwrap()),
                )
                .unwrap();
                if let Ok((outcome, fx)) = e.present_analysis(result) {
                    effects(&fx);
                    refusal = stale;
                    assert_eq!(outcome == AnalysisOutcome::Stale, stale);
                    if stale {
                        assert!(fx.mutations.is_empty() && fx.event.is_none());
                    }
                } else {
                    assert!(!stale);
                }
            }
            10 => {
                effects(&e.apply(Input::Request(R::Submit)).unwrap());
            }
            11..=13 => {
                let disposition = match c[0] % 24 {
                    11 => ValidationDisposition::Complete,
                    12 => ValidationDisposition::Incomplete,
                    _ => ValidationDisposition::Invalid(vec![
                        Diagnostic::new("invalid", Some(range))
                            .unwrap_or_else(|_| Diagnostic::new("invalid", None).unwrap()),
                    ]),
                };
                let complete = matches!(disposition, ValidationDisposition::Complete);
                let incomplete = matches!(disposition, ValidationDisposition::Incomplete);
                let result = ValidationResult::new(s.revision(), disposition).unwrap();
                if let Ok((outcome, fx)) = e.apply_validation(result) {
                    effects(&fx);
                    refusal = stale;
                    assert_eq!(outcome == AnalysisOutcome::Stale, stale);
                    if stale {
                        assert!(fx.mutations.is_empty() && fx.event.is_none());
                    } else if complete {
                        assert_eq!(fx.event, Some(Event::Submitted(before.text().into())));
                        assert_ne!(before.revision(), e.editor.revision());
                    } else if incomplete {
                        assert!(fx.event.is_none());
                        let mut expected = before.text().to_owned();
                        expected.insert(before.cursor(), '\n');
                        assert_eq!(e.editor.text(), expected);
                    } else {
                        assert!(fx.event.is_none());
                        assert_eq!(before.revision(), e.editor.revision());
                    }
                } else {
                    assert!(!stale);
                }
            }
            14 => {
                if let Ok((_, fx)) = e.completion_action(CompletionAction::Next) {
                    effects(&fx);
                }
                assert_eq!(before.revision(), e.editor.revision());
            }
            15 => {
                if let Ok((_, fx)) = e.completion_action(CompletionAction::Accept) {
                    effects(&fx);
                }
            }
            16 => {
                if let Ok((_, fx)) = e.completion_action(CompletionAction::Dismiss) {
                    effects(&fx);
                }
                assert_eq!(before.revision(), e.editor.revision());
            }
            17 => {
                effects(
                    &e.apply(Input::Resize(
                        [2, 20, 40, 80, 132][c[2] as usize % 5],
                        2 + c[3] as usize % 30,
                    ))
                    .unwrap(),
                );
                assert_eq!(before.revision(), e.editor.revision());
            }
            18 => {
                effects(&e.external_output(Role::Success, "host output\n").unwrap());
                assert_eq!(before.revision(), e.editor.revision());
            }
            19 => {
                effects(&e.close());
                assert_eq!(before.revision(), e.editor.revision());
            }
            20 => {
                effects(&e.apply(Input::Request(R::Interrupt)).unwrap());
                assert_ne!(before.revision(), e.editor.revision());
            }
            21 => {
                effects(&e.apply(Input::TransportEof).unwrap());
                assert_ne!(before.revision(), e.editor.revision());
            }
            22 => {
                e.editor.admit_history("history\n界").unwrap();
                effects(&e.apply(Input::Edit(E::HistoryPrevious)).unwrap());
            }
            _ => {
                effects(&e.apply(Input::Edit(E::HistoryNext)).unwrap());
            }
        }
        check_editor(&e.editor);
        if refusal {
            assert_eq!(
                (before.text(), before.cursor(), before.revision()),
                (e.editor.text(), e.editor.cursor(), e.editor.revision())
            );
            assert_eq!(selection, e.completion_selection());
            assert_eq!(old_analysis, format!("{:?}", e.analysis_presentation()));
            assert_eq!(old_diagnostics, format!("{:?}", e.diagnostics()));
            assert!(e.flush().mutations.is_empty());
        }
        if before.revision() != e.editor.revision() {
            assert!(e.analysis_presentation().is_none());
            assert!(e.diagnostics().is_none());
            assert!(e.completion_selection().is_none());
        }
    }
}

/// Safe host fields, deterministic document/layout, bounded viewport and geometry.
pub fn geometry_case(data: &[u8]) {
    let text = String::from_utf8_lossy(&data[..data.len().min(8192)]);
    let width = [2, 3, 20, 40, 80, 132][data.first().copied().unwrap_or(0) as usize % 6];
    let height = 2 + data.get(1).copied().unwrap_or(0) as usize % 30;
    // Constructors are independent admission domains; arbitrary bytes exercise each.
    let _ = Hint::new(&text, Role::Dim);
    let _ = Diagnostic::new(&text, None);
    let _ = CompletionCandidate::new(0..0, &text, &text).and_then(|c| c.with_annotation(&text));
    let prompt = Prompt::new(&text).unwrap_or_else(|_| Prompt::new("test").unwrap());
    if let Ok(t) = Text::new(&text) {
        for block in [
            Block::Heading {
                level: 1,
                text: t.clone(),
            },
            Block::Paragraph(t.clone()),
        ] {
            if let Ok(d) = Document::new(vec![block]) {
                for plain in [true, false] {
                    let theme = Theme::new(!plain, false, None);
                    let a = d.render(width, theme).unwrap();
                    assert_eq!(a, d.render(width, theme).unwrap());
                    if plain {
                        assert!(!a.contains('\x1b'));
                    }
                    assert!(a.len() < 2_000_000);
                }
            }
        }
    }
    let mut e = Editor::new(8192, 0);
    if e.insert(&text).is_ok() {
        for cursor in [0, text.len() / 2, text.len()] {
            let position = e
                .text()
                .grapheme_indices(true)
                .map(|(i, _)| i)
                .chain([e.text().len()])
                .find(|i| *i >= cursor)
                .unwrap();
            e.replace(position..position, "").unwrap();
            let f = Frame::new(&e, &prompt, width, height);
            let g = Frame::new(&e, &prompt, width, height);
            assert_eq!(format!("{f:?}"), format!("{g:?}"));
            assert!(f.lines.len() <= height);
            assert!(f.cursor.col < width);
            assert!(f.cursor.row < height);
            assert_eq!(f.source_cursor, e.cursor());
            assert_eq!(f.source_len, e.text().len());
            assert!(f.widths.iter().all(|n| *n <= width));
        }
    }
}
