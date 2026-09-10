//! Public data contract, independent of a terminal, parser or scheduler.
use replai::{
    CompletionCandidate as Candidate, CompletionError, CompletionSet, EditError, Editor,
    MAX_COMPLETION_BYTES, MAX_COMPLETION_CANDIDATES, MAX_COMPLETION_FIELD_BYTES,
};
#[test]
fn distinct_insertion_labels_annotations_ranges_and_duplicates() {
    fn transferable<T: Send + Sync>() {}
    transferable::<CompletionSet>();
    let editor = Editor::new(1024, 0);
    let c = Candidate::new(0..0, "src/core.rs ", "core.rs")
        .unwrap()
        .with_annotation("Core source 界 👩‍💻")
        .unwrap();
    let set = CompletionSet::new(editor.revision(), vec![c.clone(), c]).unwrap();
    assert_eq!(set.candidates()[0], set.candidates()[1]);
    assert_eq!(set.candidates()[0].replacement(), "src/core.rs ");
    assert_eq!(set.candidates()[0].label(), "core.rs");
    assert_eq!(set.candidates()[0].range(), 0..0);
    assert_eq!(set.revision(), editor.revision());
}
#[test]
fn injection_rejection_keeps_display_and_editor_character_sets_distinct() {
    for text in [
        "\x1b[2J",
        "\x1b]52;c;payload\x07",
        "\x1bPdata\x1b\\",
        "\r",
        "\0",
        "\x7f",
        "\u{85}",
        "\u{202e}spoof",
        "\u{2066}spoof",
        "\u{200b}",
        "\u{feff}",
    ] {
        assert!(matches!(
            Candidate::new(0..0, text, "label"),
            Err(CompletionError::Edit(EditError::InvalidText))
        ));
        assert!(matches!(
            Candidate::new(0..0, "insert", text),
            Err(CompletionError::InvalidDisplay)
        ));
        assert!(matches!(
            Candidate::new(0..0, "insert", "label")
                .unwrap()
                .with_annotation(text),
            Err(CompletionError::InvalidDisplay)
        ));
    }
    for text in ["\n", "\t"] {
        assert!(Candidate::new(0..0, text, "label").is_ok());
        assert!(Candidate::new(0..0, "insert", text).is_err());
    }
    for text in ["界", "e\u{301}", "👩‍💻", "♥️", "مرحبا", "[31m literal"] {
        assert!(Candidate::new(0..0, text, text).is_ok());
    }
    assert!(Candidate::new(0..0, "", "").is_err());
}
#[test]
fn limits_are_checked_without_partial_activation() {
    let e = Editor::new(1024, 0);
    let c = Candidate::new(0..0, "x", "x").unwrap();
    assert!(CompletionSet::new(e.revision(), vec![c.clone(); MAX_COMPLETION_CANDIDATES]).is_ok());
    assert!(matches!(
        CompletionSet::new(e.revision(), vec![c; MAX_COMPLETION_CANDIDATES + 1]),
        Err(CompletionError::Limit)
    ));
    let field = "x".repeat(MAX_COMPLETION_FIELD_BYTES);
    assert!(Candidate::new(0..0, &field, "label").is_ok());
    assert!(matches!(
        Candidate::new(0..0, &(field.clone() + "x"), "label"),
        Err(CompletionError::Limit)
    ));
    let c = Candidate::new(0..0, &field, &field).unwrap();
    assert!(matches!(
        CompletionSet::new(
            e.revision(),
            vec![c; MAX_COMPLETION_BYTES / (2 * field.len()) + 1]
        ),
        Err(CompletionError::Limit)
    ));
}
