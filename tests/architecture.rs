//! Dependency-direction guard for the independently executable engine.
#[test]
fn state_and_geometry_do_not_import_resources_protocol_or_scheduling() {
    for (name, source) in [
        ("core", include_str!("../src/core.rs")),
        ("completion", include_str!("../src/completion.rs")),
        ("document", include_str!("../src/document.rs")),
        ("engine", include_str!("../src/engine.rs")),
        ("actions", include_str!("../src/actions.rs")),
        ("render", include_str!("../src/render.rs")),
    ] {
        for forbidden in [
            "std::os",
            "rustix",
            "libc::",
            "std::env",
            "std::time",
            "std::thread",
            "AtomicBool",
            "crate::terminal",
            "crate::system",
            "crate::protocol",
            "crate::input",
            "\\x1b",
        ] {
            assert!(!source.contains(forbidden), "{name} depends on {forbidden}");
        }
    }
    let input = include_str!("../src/input.rs");
    assert!(!input.contains("Editor"));
    assert!(!input.contains("std::time"));
    let system = include_str!("../src/system.rs");
    for forbidden in ["Editor", "Prompt", "Event::", "Decoder", "\\x1b"] {
        assert!(
            !system.contains(forbidden),
            "resource backend owns {forbidden}"
        );
    }
    let layout = include_str!("../src/presentation.rs")
        .split("#[cfg(test)]")
        .next()
        .unwrap();
    assert!(
        !layout.contains("\\x1b"),
        "layout serializes terminal escapes"
    );
    assert!(
        !layout.contains("std::env"),
        "layout discovers environment policy"
    );
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains("unsafe_code = \"forbid\""));
    assert!(manifest.contains(
        "[target.'cfg(any(target_os = \"linux\", target_os = \"macos\"))'.dependencies]"
    ));
}

#[test]
fn public_state_and_events_exist_without_acquiring_a_system_terminal() {
    use replai::{Editor, Event, Interaction};
    let mut first = Interaction::new(Editor::new(64, 2));
    let mut second = Interaction::new(Editor::new(64, 2));
    first.editor_mut().unwrap().insert("界").unwrap();
    second.editor_mut().unwrap().insert("independent").unwrap();
    assert!(!first.is_open() && !second.is_open());
    assert_eq!(first.editor().text(), "界");
    assert_ne!(Event::Interrupted, Event::EndOfInput);
}
