//! Portable capability/width oracle: synthetic evidence is not OS qualification.
use replai::{
    Block, Degradation, Document, Error, FeaturePolicy as P, FeatureSupport as S,
    InteractionRequirements as R, Readiness, ResizeDelivery, TerminalConfig, TerminalFacts,
    TerminalRealization, Text, Theme, WidthPolicy,
};

fn config() -> TerminalConfig {
    TerminalConfig {
        facts: TerminalFacts::assumed_vt(),
        styling: P::Preferred,
        bracketed_paste: P::Preferred,
        theme: Theme::new(true, false, None),
    }
}
#[test]
fn explicit_profile_matrix_preserves_facts_and_observes_degradation() {
    let full = TerminalRealization::interactive((80, 24));
    for support in [S::Supported, S::Assumed, S::Unknown, S::Unavailable] {
        for policy in [P::Disabled, P::Preferred, P::Required] {
            let mut c = config();
            c.facts.styling = support;
            c.styling = policy;
            let result = c.resolve_for(full, R::Editing);
            if policy == P::Required && matches!(support, S::Unknown | S::Unavailable) {
                assert!(matches!(result, Err(Error::CapabilityMismatch(_))));
                continue;
            }
            let (theme, snapshot) = result.unwrap();
            assert_eq!(snapshot.facts.styling, support);
            let doc = Document::new(vec![Block::Heading {
                level: 1,
                text: Text::new("Facts").unwrap(),
            }])
            .unwrap();
            let output = doc.render(40, theme).unwrap();
            if policy == P::Disabled || matches!(support, S::Unknown | S::Unavailable) {
                assert!(!snapshot.features.styling);
                assert_eq!(output, "# Facts\n");
                assert_ne!(snapshot.styling_degradation, Degradation::None);
            } else {
                assert!(snapshot.features.styling);
                assert!(output.contains('\x1b'));
            }
        }
    }
    for support in [S::Supported, S::Assumed, S::Unknown, S::Unavailable] {
        let mut c = config();
        c.facts.bracketed_paste = support;
        let (_, s) = c.resolve_for(full, R::Editing).unwrap();
        assert_eq!(
            s.features.bracketed_paste,
            matches!(support, S::Supported | S::Assumed)
        );
        c.bracketed_paste = P::Required;
        assert_eq!(
            c.resolve_for(full, R::Editing).is_ok(),
            s.features.bracketed_paste
        );
    }
}
#[test]
fn required_mechanics_and_stronger_resource_evidence_cannot_be_overridden() {
    let full = TerminalRealization::interactive((80, 24));
    let c = config();
    for field in 0..6 {
        let mut r = full;
        match field {
            0 => r.input = S::Unavailable,
            1 => r.output = S::Unavailable,
            2 => r.restoration = S::Unknown,
            3 => r.dimensions = None,
            4 => r.dimensions = Some((1, 24)),
            _ => r.readiness = Readiness::Unavailable,
        }
        assert!(matches!(
            c.resolve_for(r, R::Editing),
            Err(Error::CapabilityMismatch(_))
        ));
    }
    for property in 0..2 {
        let mut c = c;
        if property == 0 {
            c.facts.cursor = S::Unknown;
        } else {
            c.facts.erase = S::Unavailable;
        }
        assert!(c.resolve_for(full, R::Editing).is_err());
    }
    let mut c = c;
    c.theme = Theme::new(true, true, None);
    c.styling = P::Required;
    assert!(c.resolve_for(full, R::Editing).is_err());
}
#[test]
fn readiness_query_notification_and_presentation_are_independent() {
    let mut r = TerminalRealization::interactive((40, 12));
    r.readiness = Readiness::BackendManaged;
    assert!(config().resolve_for(r, R::Editing).is_ok());
    assert!(config().resolve_for(r, R::Driven).is_err());
    r.readiness = Readiness::Waitable;
    let (_, snapshot) = config().resolve_for(r, R::Driven).unwrap();
    assert_eq!(snapshot.resize, ResizeDelivery::PollOrHostNotification);
    r.dimension_query = S::Unavailable;
    assert_eq!(
        config().resolve_for(r, R::Driven).unwrap().1.resize,
        ResizeDelivery::Fixed
    );
    let mut c = config();
    c.facts = TerminalFacts::from_term_hint(Some("dumb"));
    let (theme, snapshot) = c
        .resolve_for(TerminalRealization::captured(), R::Presentation)
        .unwrap();
    assert_eq!(snapshot.resize, ResizeDelivery::Fixed);
    assert!(!snapshot.features.styling);
    assert!(!snapshot.features.bracketed_paste);
    let doc = Document::new(vec![Block::Paragraph(Text::new("plain 界").unwrap())]).unwrap();
    assert_eq!(doc.render(20, theme).unwrap(), "plain 界\n");
    assert!(c.resolve_for(r, R::Editing).is_err());
}
#[test]
fn term_hints_never_claim_discovery_and_width_is_separate_from_segmentation() {
    assert_eq!(
        TerminalFacts::from_term_hint(Some("xterm")).cursor,
        S::Assumed
    );
    for hint in [None, Some(""), Some("dumb")] {
        assert_eq!(TerminalFacts::from_term_hint(hint).cursor, S::Unknown);
    }
    for (text, cells) in [
        ("ASCII", 5),
        ("e\u{301}", 1),
        ("界", 2),
        ("👩‍💻", 2),
        ("👨‍👩‍👧‍👦", 2),
        ("Ω·", 2),
    ] {
        assert_eq!(
            WidthPolicy::UnicodeNarrow.measure(text).unwrap(),
            cells,
            "{text}"
        );
    }
    for control in ["a\tb", "a\nb", "\x1b[31m"] {
        assert!(WidthPolicy::UnicodeNarrow.measure(control).is_err());
    }
}

#[test]
fn environment_policy_probe() {
    if std::env::var_os("REPLAI_CAPABILITY_CHILD").is_none() {
        return;
    }
    let mut c = TerminalConfig::from_environment();
    let hint = std::env::var("TERM").ok();
    assert_eq!(c.facts, TerminalFacts::from_term_hint(hint.as_deref()));
    let no_color = std::env::var_os("NO_COLOR").is_some();
    assert_eq!(c.styling, if no_color { P::Disabled } else { P::Preferred });
    // Stronger explicit host evidence replaces TERM, but never NO_COLOR policy.
    c.facts = TerminalFacts {
        styling: S::Supported,
        ..TerminalFacts::assumed_vt()
    };
    let (_, s) = c
        .resolve_for(TerminalRealization::interactive((80, 24)), R::Editing)
        .unwrap();
    assert_eq!(s.facts.styling, S::Supported);
    assert_eq!(s.features.styling, !no_color);
}
#[test]
fn environment_policy_isolated_from_test_process_and_stronger_host_evidence() {
    for term in [None, Some("dumb"), Some("xterm-256color")] {
        for no_color in [false, true] {
            let mut command = std::process::Command::new(std::env::current_exe().unwrap());
            command
                .args(["--exact", "environment_policy_probe"])
                .env("REPLAI_CAPABILITY_CHILD", "1")
                .env_remove("TERM")
                .env_remove("NO_COLOR");
            if let Some(term) = term {
                command.env("TERM", term);
            }
            if no_color {
                command.env("NO_COLOR", "");
            }
            let output = command.output().unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stdout)
            );
        }
    }
}
