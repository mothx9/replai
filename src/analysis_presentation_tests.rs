use super::*;
use crate::{
    AnalysisOutcome as A, AnalysisPresentation as P, AnalysisSpan as S, Diagnostic, Hint,
    SubmissionPolicy, Theme, ValidationDisposition, ValidationResult,
};
fn result(e: &Engine) -> P {
    P::new(
        e.editor.revision(),
        vec![S::new(0..e.editor.text().len(), Role::Accent).unwrap()],
        Some(Hint::new("suffix 界", Role::Dim).unwrap()),
    )
    .unwrap()
}
fn engine(text: &str) -> Engine {
    let mut e = Engine::new(Editor::new(1_100_000, 8));
    e.editor.insert(text).unwrap();
    e.set_submission_policy(SubmissionPolicy::Validated)
        .unwrap();
    e.start(Prompt::new("d").unwrap(), (80, 24)).unwrap();
    e
}
fn encode(fx: &Effects, plain: bool) -> String {
    crate::protocol::encode(&fx.mutations, Theme::new(true, plain, Some("xterm")))
}
#[test]
fn atomic_replacement_clear_and_stale_zero_effects() {
    let mut e = engine("e\u{301}界");
    let p = result(&e);
    let snapshot = e.editor.analysis_snapshot();
    assert_eq!(e.present_analysis(p.clone()).unwrap().0, A::Applied);
    assert_eq!(e.editor.analysis_snapshot(), snapshot);
    let bad = P::new(
        snapshot.revision(),
        vec![S::new(0..1, Role::Error).unwrap()],
        None,
    )
    .unwrap();
    assert!(e.present_analysis(bad).is_err());
    assert_eq!(e.analysis_presentation(), Some(&p));
    e.apply(Input::Edit(EditCommand::Left)).unwrap();
    assert!(e.analysis_presentation().is_none());
    e.apply(Input::Edit(EditCommand::Right)).unwrap();
    assert_eq!(e.editor.text(), snapshot.text());
    let (a, fx) = e.present_analysis(p).unwrap();
    assert_eq!(a, A::Stale);
    assert!(fx.mutations.is_empty());
    e.present_analysis(result(&e)).unwrap();
    e.present_analysis(P::new(e.editor.revision(), vec![], None).unwrap())
        .unwrap();
    assert!(e.analysis_presentation().is_none());
    let r = e.editor.revision();
    e.close();
    assert_eq!(r, e.editor.revision());
    assert!(
        e.present_analysis(P::new(r, vec![], None).unwrap())
            .is_err()
    );
    e.start(Prompt::new("d").unwrap(), (80, 24)).unwrap();
    assert!(e.analysis_presentation().is_none());
}
#[test]
fn completion_validation_output_resize_and_submission_compose() {
    for plain in [false, true] {
        let mut e = engine("bu");
        let p = result(&e);
        let r = p.revision();
        let fx = e.present_analysis(p.clone()).unwrap().1;
        assert!(encode(&fx, plain).contains("[~suffix"));
        for width in [20, 40, 80, 132] {
            e.apply(Input::Resize(width, 12)).unwrap();
            assert_eq!(e.editor.revision(), r);
        }
        e.external_output(Role::Dim, "notice").unwrap();
        assert_eq!(e.analysis_presentation(), Some(&p));
        let c = crate::CompletionCandidate::new(0..2, "build", "build").unwrap();
        let fx = e
            .present_completions(crate::CompletionSet::new(r, vec![c]).unwrap())
            .unwrap()
            .1;
        assert!(!encode(&fx, plain).contains("[~suffix"));
        assert_eq!(e.analysis_presentation(), Some(&p));
        let fx = e
            .completion_action(crate::CompletionAction::Dismiss)
            .unwrap()
            .1;
        assert!(encode(&fx, plain).contains("[~suffix"));
        e.apply(Input::Request(Request::Submit)).unwrap();
        let fx = e
            .apply_validation(
                ValidationResult::new(
                    r,
                    ValidationDisposition::Invalid(vec![Diagnostic::new("bad", None).unwrap()]),
                )
                .unwrap(),
            )
            .unwrap()
            .1;
        assert!(encode(&fx, plain).contains("Invalid input"));
        assert_eq!(e.analysis_presentation(), Some(&p));
        e.apply(Input::Request(Request::Submit)).unwrap();
        e.apply_validation(ValidationResult::new(r, ValidationDisposition::Incomplete).unwrap())
            .unwrap();
        assert!(e.analysis_presentation().is_none());
        e.present_analysis(result(&e)).unwrap();
        let snapshot = e.editor.analysis_snapshot();
        e.apply(Input::Request(Request::Submit)).unwrap();
        let fx = e
            .apply_validation(
                ValidationResult::new(snapshot.revision(), ValidationDisposition::Complete)
                    .unwrap(),
            )
            .unwrap()
            .1;
        assert_eq!(fx.event, Some(Event::Submitted("bu\n".into())));
        assert!(e.analysis_presentation().is_none());
    }
}
#[test]
fn spans_preserve_geometry_and_hint_never_wraps_or_becomes_canonical() {
    use crate::presentation::{Frame, Run};
    use unicode_segmentation::UnicodeSegmentation;
    for text in ["a界e\u{301}👩‍💻x", "line\n界\tend", "x\n"] {
        let mut e = engine(text);
        let p = result(&e);
        for width in [2, 20, 40, 80, 132] {
            for pos in text
                .grapheme_indices(true)
                .map(|(i, _)| i)
                .chain(std::iter::once(text.len()))
            {
                e.editor.replace(pos..pos, "").unwrap();
                let prompt = Prompt::new("d").unwrap();
                let a = Frame::new(&e.editor, &prompt, width, 12);
                let mut b = Frame::analyzed(&e.editor, &prompt, width, 12, Some(&p));
                assert_eq!(a.cursor, b.cursor);
                assert_eq!(a.widths, b.widths);
                let strip = |f: &Frame| {
                    f.lines
                        .iter()
                        .map(|l| {
                            l.0.iter()
                                .filter_map(|r| match r {
                                    Run::Text(t) => Some(t.as_str()),
                                    _ => None,
                                })
                                .collect::<String>()
                        })
                        .collect::<Vec<_>>()
                };
                assert_eq!(strip(&a), strip(&b));
                p.append_hint(&mut b);
                assert_eq!(a.cursor, b.cursor);
                assert_eq!(a.lines.len(), b.lines.len());
                if pos != text.len() {
                    assert_eq!(strip(&a), strip(&b));
                }
                assert!(b.widths.iter().all(|w| *w <= width));
            }
        }
    }
}
#[test]
fn long_generated_revision_model_and_large_viewports() {
    for lines in [10, 100, 1000, 16000] {
        let text = "row 界 e\u{301} 👩‍💻\n".repeat(lines);
        let mut e = engine(&text);
        let p = result(&e);
        let fx = e.present_analysis(p.clone()).unwrap().1;
        assert!(fx.mutations.len() < 250);
        for width in [20, 40, 80, 132] {
            e.apply(Input::Resize(width, 12)).unwrap();
            assert_eq!(e.analysis_presentation(), Some(&p));
        }
    }
    let mut e = engine("abc");
    let mut seed = 7_u64;
    for _ in 0..10000 {
        if e.editor.text().is_empty() {
            e.apply(Input::Text("x".into())).unwrap();
        }
        let p = result(&e);
        e.present_analysis(p.clone()).unwrap();
        let snapshot = e.editor.analysis_snapshot();
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let input = match (seed >> 32) % 7 {
            0 => Input::Text("x".into()),
            1 => Input::Edit(EditCommand::Left),
            2 => Input::Edit(EditCommand::Right),
            3 => Input::Edit(EditCommand::Backspace),
            4 => Input::Edit(EditCommand::Home),
            5 => Input::Resize(40, 12),
            _ => Input::Edit(EditCommand::Delete),
        };
        e.apply(input).unwrap();
        let changed = e.editor.revision() != snapshot.revision();
        assert_eq!(e.analysis_presentation().is_none(), changed);
        let now = e.editor.analysis_snapshot();
        let (a, fx) = e.present_analysis(p).unwrap();
        assert_eq!(a, if changed { A::Stale } else { A::Applied });
        assert_eq!(e.editor.analysis_snapshot(), now);
        if changed {
            assert!(fx.mutations.is_empty());
        }
    }
}
