//! Portable validation bounds, policy and logical multiline geometry.
use replai::*;
#[test]
fn safe_bounded_messages_and_ranges() {
    for bad in [
        "",
        "\x1b[2J",
        "\x1b]52;c;x\x07",
        "\r",
        "\t",
        "\n",
        "\u{202e}fake",
        "\u{2066}x",
    ] {
        assert!(matches!(
            Diagnostic::new(bad, None),
            Err(ValidationError::InvalidMessage)
        ));
    }
    assert!(Diagnostic::new("界e\u{301}👩‍💻", Some(0..0)).is_ok());
    assert!(matches!(
        Diagnostic::new(&"x".repeat(MAX_DIAGNOSTIC_BYTES + 1), None),
        Err(ValidationError::Limit)
    ));
    let revision = Editor::new(10, 0).revision();
    let d = Diagnostic::new("x", None).unwrap();
    assert!(matches!(
        ValidationResult::new(
            revision,
            ValidationDisposition::Invalid(vec![d; MAX_DIAGNOSTICS + 1])
        ),
        Err(ValidationError::Limit)
    ));
    let d = Diagnostic::new(&"x".repeat(MAX_DIAGNOSTIC_BYTES), None).unwrap();
    assert!(matches!(
        ValidationResult::new(
            revision,
            ValidationDisposition::Invalid(vec![d; MAX_DIAGNOSTICS])
        ),
        Err(ValidationError::Limit)
    ));
}
#[test]
fn logical_vertical_geometry_and_revision() {
    let mut e = Editor::new(4096, 2);
    e.insert("界e\u{301}\n12345\n\tZ").unwrap();
    e.home();
    e.right();
    let rev = e.revision();
    assert!(!e.line_up());
    assert_eq!(e.revision(), rev);
    assert!(e.line_down());
    assert_eq!(e.cursor(), 9);
    assert_ne!(e.revision(), rev);
    assert!(e.line_down());
    assert_eq!(e.cursor(), 13); // tab is wider than target column
    let r = e.revision();
    assert!(!e.line_down());
    assert_eq!(e.revision(), r);
}
#[test]
fn policy_and_results_are_portable_without_a_terminal() {
    let mut i = Interaction::new(Editor::new(64, 2));
    assert_eq!(i.submission_policy(), SubmissionPolicy::Direct);
    i.set_submission_policy(SubmissionPolicy::Validated)
        .unwrap();
    assert_eq!(i.submission_policy(), SubmissionPolicy::Validated);
    assert!(i.diagnostics().is_none());
    fn movable<T: Send + Sync>() {}
    movable::<ValidationResult>();
    movable::<AnalysisSnapshot>();
}
