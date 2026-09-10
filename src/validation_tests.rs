use super::*;
use crate::{
    AnalysisOutcome as A, Diagnostic, SubmissionPolicy, ValidationDisposition as V,
    ValidationError, ValidationResult,
};
fn engine(text: &str) -> Engine {
    let mut e = Engine::new(Editor::new(1_048_576, 10));
    e.editor.insert(text).unwrap();
    e.set_submission_policy(SubmissionPolicy::Validated)
        .unwrap();
    e.start(Prompt::new("demo").unwrap(), (80, 24)).unwrap();
    e
}
fn request(e: &mut Engine) -> crate::AnalysisSnapshot {
    match e
        .apply(Input::Request(Request::Submit))
        .unwrap()
        .event
        .unwrap()
    {
        Event::SubmissionRequested(s) => s,
        other => panic!("{other:?}"),
    }
}
fn respond(e: &mut Engine, revision: crate::DraftRevision, d: V) -> (A, Effects) {
    e.apply_validation(ValidationResult::new(revision, d).unwrap())
        .unwrap()
}
#[test]
fn dispositions_are_atomic_and_requests_are_consumed() {
    let mut e = engine("begin {");
    let s = request(&mut e);
    let (_, fx) = respond(&mut e, s.revision(), V::Incomplete);
    assert!(fx.event.is_none());
    assert_eq!(e.editor.text(), "begin {\n");
    assert_ne!(s.revision(), e.editor.revision());
    assert!(e.is_open());
    let s = request(&mut e);
    let (_, fx) = respond(
        &mut e,
        s.revision(),
        V::Invalid(vec![Diagnostic::new("close bracket", Some(6..7)).unwrap()]),
    );
    assert!(fx.event.is_none());
    assert_eq!(s.revision(), e.editor.revision());
    assert_eq!(e.diagnostics().unwrap()[0].message(), "close bracket");
    assert!(matches!(
        e.apply_validation(ValidationResult::new(s.revision(), V::Complete).unwrap()),
        Err(ValidationError::NoRequest)
    ));
    let s = request(&mut e);
    let (_, fx) = respond(&mut e, s.revision(), V::Complete);
    assert_eq!(fx.event, Some(Event::Submitted("begin {\n".into())));
    assert!(!e.is_open());
    let (out, fx) = respond(&mut e, s.revision(), V::Complete);
    assert_eq!(out, A::Stale);
    assert!(fx.event.is_none() && fx.mutations.is_empty());
}
#[test]
fn stale_is_silent_for_every_disposition_and_cursor_change() {
    for cursor in [false, true] {
        for disposition in [
            V::Complete,
            V::Incomplete,
            V::Invalid(vec![Diagnostic::new("bad", Some(999..1000)).unwrap()]),
        ] {
            let mut e = engine("abc");
            let s = request(&mut e);
            e.apply(if cursor {
                Input::Edit(EditCommand::Left)
            } else {
                Input::Text("x".into())
            })
            .unwrap();
            let text = e.editor.text().to_owned();
            let position = e.editor.cursor();
            let rev = e.editor.revision();
            let (out, fx) = respond(&mut e, s.revision(), disposition);
            assert_eq!(out, A::Stale);
            assert!(fx.mutations.is_empty() && fx.event.is_none());
            assert_eq!(
                (e.editor.text(), e.editor.cursor(), e.editor.revision()),
                (text.as_str(), position, rev)
            );
            assert!(e.diagnostics().is_none());
        }
    }
}
#[test]
fn invalid_range_and_capacity_preserve_request_then_valid_result_applies() {
    let mut e = engine("e\u{301}界");
    let s = request(&mut e);
    for range in [1..2, 0..999, 3..4] {
        let result = ValidationResult::new(
            s.revision(),
            V::Invalid(vec![Diagnostic::new("safe", Some(range)).unwrap()]),
        )
        .unwrap();
        assert!(matches!(
            e.apply_validation(result),
            Err(ValidationError::Edit(crate::EditError::InvalidRange))
        ));
        assert_eq!(e.editor.revision(), s.revision());
        assert!(e.diagnostics().is_none());
    }
    respond(&mut e, s.revision(), V::Complete);
    let mut e = Engine::new(Editor::new(1, 0));
    e.editor.insert("x").unwrap();
    e.set_submission_policy(SubmissionPolicy::Validated)
        .unwrap();
    e.start(Prompt::new("d").unwrap(), (20, 4)).unwrap();
    let s = request(&mut e);
    assert!(matches!(
        e.apply_validation(ValidationResult::new(s.revision(), V::Incomplete).unwrap()),
        Err(ValidationError::Edit(crate::EditError::Capacity))
    ));
    assert_eq!(e.editor.revision(), s.revision());
    respond(&mut e, s.revision(), V::Complete);
}
#[test]
fn diagnostics_survive_output_resize_but_not_revision_change_or_close() {
    let mut e = engine("first\n界 second\nthird");
    let s = request(&mut e);
    respond(
        &mut e,
        s.revision(),
        V::Invalid(vec![
            Diagnostic::new("expected closing delimiter", None).unwrap(),
        ]),
    );
    for width in [20, 40, 80, 132] {
        e.apply(Input::Resize(width, 8)).unwrap();
        e.external_output(Role::Dim, "notice").unwrap();
        assert_eq!(e.editor.revision(), s.revision());
        assert!(e.diagnostics().is_some());
    }
    e.apply(Input::Edit(EditCommand::Left)).unwrap();
    assert!(e.diagnostics().is_none());
    let s = request(&mut e);
    e.close();
    e.start(Prompt::new("d").unwrap(), (80, 24)).unwrap();
    assert_eq!(e.editor.revision(), s.revision());
    assert!(matches!(
        e.apply_validation(ValidationResult::new(s.revision(), V::Complete).unwrap()),
        Err(ValidationError::NoRequest)
    ));
}
#[test]
fn completion_acceptance_precedes_validation_and_history_remains_navigable() {
    let mut e = engine("b");
    let rev = e.editor.revision();
    e.present_completions(
        crate::CompletionSet::new(
            rev,
            vec![crate::CompletionCandidate::new(0..1, "begin {", "begin").unwrap()],
        )
        .unwrap(),
    )
    .unwrap();
    assert!(
        e.apply(Input::Request(Request::Submit))
            .unwrap()
            .event
            .is_none()
    );
    assert_eq!(e.editor.text(), "begin {");
    let s = request(&mut e);
    respond(&mut e, s.revision(), V::Incomplete);
    e.editor.admit_history("old\nentry").unwrap();
    e.apply(Input::Edit(EditCommand::HistoryPrevious)).unwrap();
    assert_eq!(e.editor.cursor(), 0);
    e.apply(Input::Edit(EditCommand::HistoryPrevious)).unwrap();
    assert_eq!(e.editor.text(), "old\nentry");
    e.apply(Input::Edit(EditCommand::HistoryNext)).unwrap();
    assert_eq!(e.editor.text(), "begin {\n");
}
#[test]
fn long_generated_model_never_revives_stale_submission() {
    let mut e = engine("");
    let mut n = 77u64;
    for _ in 0..10000 {
        if e.editor.text().len() > 300 {
            e.complete(0..e.editor.text().len(), "").unwrap();
        }
        let s = request(&mut e);
        n = n.wrapping_mul(6364136223846793005).wrapping_add(1);
        match (n >> 32) % 6 {
            0 => {
                respond(&mut e, s.revision(), V::Incomplete);
            }
            1 => {
                respond(&mut e, s.revision(), V::Invalid(vec![]));
            }
            2 => {
                e.apply(Input::Text("界".into())).unwrap();
            }
            3 => {
                e.apply(Input::Edit(EditCommand::Home)).unwrap();
            }
            4 => {
                e.apply(Input::Edit(EditCommand::End)).unwrap();
            }
            _ => {
                e.apply(Input::Text("ab\ncd".into())).unwrap();
            }
        }
        if e.editor.revision() != s.revision() {
            let current = e.editor.analysis_snapshot();
            let (a, fx) = respond(&mut e, s.revision(), V::Complete);
            assert_eq!(a, A::Stale);
            assert!(fx.mutations.is_empty());
            assert_eq!(current, e.editor.analysis_snapshot());
        }
    }
}
#[test]
fn substantial_multiline_layout_is_bounded_and_unicode_vertical_is_grapheme_safe() {
    for lines in [10, 100, 1000, 16000] {
        let text = "界e\u{301}👩‍💻\t line\n".repeat(lines);
        let mut e = engine(&text);
        for pos in [0, text.len() / 2, text.len()] {
            let pos = text[..pos].rfind('\n').map_or(0, |p| p + 1);
            e.complete(pos..pos, "").unwrap();
            for width in [20, 40, 80, 132] {
                e.apply(Input::Resize(width, 10)).unwrap();
                let rev = e.editor.revision();
                let snapshot = e.editor.analysis_snapshot();
                e.external_output(Role::Dim, "status").unwrap();
                assert_eq!(e.editor.revision(), rev);
                assert_eq!(snapshot, e.editor.analysis_snapshot());
                let f = crate::presentation::Frame::new(
                    &e.editor,
                    &Prompt::new("demo").unwrap(),
                    width,
                    10,
                );
                assert!(f.lines.len() <= 9);
            }
            e.editor.line_up();
            e.editor.line_down();
            assert!(
                e.editor
                    .validate_replacement(&(e.editor.cursor()..e.editor.cursor()), "")
                    .is_ok()
            );
        }
    }
}
