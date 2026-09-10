//! Portable public analysis-presentation payload invariants.
use replai::*;
#[test]
fn bounds_order_and_safe_text_are_portable() {
    let revision = Editor::new(1024, 0).revision();
    assert!(AnalysisSpan::new(1..1, Role::Accent).is_err());
    assert!(AnalysisSpan::new(std::ops::Range { start: 2, end: 1 }, Role::Accent).is_err());
    for ranges in [[0..3, 2..4], [2..4, 0..2]] {
        assert!(
            AnalysisPresentation::new(
                revision,
                ranges
                    .into_iter()
                    .map(|r| AnalysisSpan::new(r, Role::Accent).unwrap())
                    .collect(),
                None
            )
            .is_err()
        );
    }
    assert!(
        AnalysisPresentation::new(
            revision,
            vec![AnalysisSpan::new(0..1, Role::Accent).unwrap(); MAX_ANALYSIS_SPANS + 1],
            None
        )
        .is_err()
    );
    for text in [
        "",
        "\x1b[31m",
        "\x1b]52;c;secret\x07",
        "\r",
        "\n",
        "\t",
        "\u{202e}",
        "\u{200b}",
    ] {
        assert!(Hint::new(text, Role::Dim).is_err(), "{text:?}");
    }
    assert!(Hint::new(&"a".repeat(MAX_HINT_BYTES + 1), Role::Dim).is_err());
    assert!(Hint::new("e\u{301}界👩‍💻", Role::Dim).is_ok());
    let p = AnalysisPresentation::new(revision, vec![], None).unwrap();
    assert_eq!(p.revision(), revision);
    fn send_sync<T: Send + Sync>() {}
    send_sync::<AnalysisPresentation>();
}
