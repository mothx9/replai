//! Real resources with explicitly configured protocol assumptions. No emulator probing.
#![cfg(any(target_os = "linux", target_os = "macos"))]
use replai::{
    Block, Degradation, Document, Editor, Error, Event, FeaturePolicy as P, FeatureSupport as S,
    Interaction, InteractionRequirements as R, Prompt, Readiness, ResizeDelivery, TerminalConfig,
    TerminalFacts, Text, Theme, WaitInterest, Wake,
};
use rustix::{
    fs::{OFlags, fcntl_getfl, fcntl_setfl},
    io::{read, write},
    termios::{Winsize, tcgetattr, tcsetwinsize},
};
#[path = "support/posix_pty.rs"]
mod pty_support;
fn resize(fd: &impl std::os::fd::AsFd, columns: u16) {
    tcsetwinsize(
        fd,
        Winsize {
            ws_col: columns,
            ws_row: 24,
            ws_xpixel: 0,
            ws_ypixel: 0,
        },
    )
    .unwrap();
}
fn drain(fd: &impl std::os::fd::AsFd) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut buffer = [0; 8192];
    while let Ok(n) = read(fd, &mut buffer) {
        if n == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..n]);
    }
    bytes
}
#[test]
fn native_profiles_observe_output_admission_and_exact_restoration() {
    let (master, slave) = pty_support::pair();
    fcntl_setfl(&master, fcntl_getfl(&master).unwrap() | OFlags::NONBLOCK).unwrap();
    let saved = format!("{:?}", tcgetattr(&slave).unwrap());
    for (name, color, paste) in [
        ("full", true, true),
        ("plain", false, true),
        ("no-paste", true, false),
        ("plain-no-paste", false, false),
    ] {
        resize(&slave, 80);
        drain(&master);
        let mut t = Interaction::new(Editor::new(1024, 5));
        let c = TerminalConfig {
            facts: TerminalFacts {
                styling: if color { S::Assumed } else { S::Unavailable },
                bracketed_paste: if paste { S::Assumed } else { S::Unavailable },
                ..TerminalFacts::assumed_vt()
            },
            styling: P::Preferred,
            bracketed_paste: P::Preferred,
            theme: Theme::new(true, false, None),
        };
        t.open_with_config(&slave, &slave, Prompt::new("caps").unwrap(), c)
            .unwrap();
        let snapshot = t.capabilities().unwrap();
        assert_eq!(snapshot.requirements, R::Driven);
        assert_eq!(snapshot.realization.input, S::Supported);
        assert_eq!(snapshot.facts.cursor, S::Assumed);
        assert_eq!(snapshot.realization.readiness, Readiness::Waitable);
        assert_eq!(snapshot.resize, ResizeDelivery::PollOrHostNotification);
        assert_eq!(snapshot.features.styling, color);
        assert_eq!(snapshot.features.bracketed_paste, paste);
        assert_eq!(
            snapshot.paste_degradation,
            if paste {
                Degradation::None
            } else {
                Degradation::Unavailable
            }
        );
        assert_ne!(format!("{:?}", tcgetattr(&slave).unwrap()), saved);
        write(&master, "e\u{301}界\x1b[D!".as_bytes()).unwrap();
        t.advance(Wake::InputReady).unwrap();
        assert_eq!((t.editor().text(), t.editor().cursor()), ("e\u{301}!界", 4));
        let doc = Document::new(vec![Block::Heading {
            level: 1,
            text: Text::new("Output").unwrap(),
        }])
        .unwrap();
        t.output_document(&doc).unwrap();
        assert_eq!((t.editor().text(), t.editor().cursor()), ("e\u{301}!界", 4));
        let bytes = drain(&master);
        let wire = String::from_utf8_lossy(&bytes);
        assert_eq!(wire.contains("\x1b[38;5;"), color);
        assert_eq!(wire.contains("\x1b[?2004h"), paste);
        let mut screen = vt100::Parser::new(24, 80, 100);
        screen.process(&bytes);
        assert!(screen.screen().contents().contains("# Output"));
        assert!(screen.screen().contents().contains("caps> e\u{301}!界"));
        assert_eq!(screen.screen().cursor_position().1, 8);
        assert_eq!(screen.screen().bracketed_paste(), paste);
        resize(&slave, 12);
        t.advance(Wake::Resize).unwrap();
        assert_eq!(
            t.capabilities().unwrap().realization.dimensions,
            Some((12, 24))
        );
        assert_eq!((t.editor().text(), t.editor().cursor()), ("e\u{301}!界", 4));
        assert_eq!(
            t.wait_interest().unwrap(),
            WaitInterest::Input { deadline: None }
        );
        t.interrupt().unwrap();
        assert_eq!(format!("{:?}", tcgetattr(&slave).unwrap()), saved);
        screen.process(&drain(&master));
        assert!(!screen.screen().bracketed_paste());
        assert!(t.capabilities().is_err());
        // Degraded ordinary editing is valid; without framing Enter means submit.
        t.editor_mut().unwrap().clear();
        t.open_with_config(&slave, &slave, Prompt::new("caps").unwrap(), c)
            .unwrap();
        let expected = if paste { "first\nsecond" } else { "first" };
        write(
            &master,
            if paste {
                b"\x1b[200~first\r\nsecond\x1b[201~\r".as_slice()
            } else {
                b"first\rsecond"
            },
        )
        .unwrap();
        assert_eq!(
            t.advance(Wake::InputReady).unwrap(),
            Some(Event::Submitted(expected.into()))
        );
        assert_eq!(format!("{:?}", tcgetattr(&slave).unwrap()), saved);
        let mut screen = vt100::Parser::new(24, 12, 100);
        screen.process(&drain(&master));
        assert!(!screen.screen().bracketed_paste());
        if !paste {
            t.open_with_config(&slave, &slave, Prompt::new("caps").unwrap(), c)
                .unwrap();
            assert_eq!(t.wait_interest().unwrap(), WaitInterest::Ready);
            t.advance(Wake::InputReady).unwrap();
            assert_eq!(t.editor().text(), "firstsecond");
            t.close().unwrap();
            assert_eq!(format!("{:?}", tcgetattr(&slave).unwrap()), saved);
            drain(&master);
        }
        eprintln!(
            "PROFILE {name}: input=Supported cursor=Assumed dimensions=80x24->12x24 readiness=Waitable styling={color} paste={paste}; draft=é!界 cursor_bytes=4 screen_col=8 output=heading; submitted={expected:?}; termios=exact paste_after_close=false"
        );
    }
    // Zero dimensions are observed, never overwritten by affirmative host facts.
    resize(&slave, 0);
    let c = TerminalConfig::compatibility(Theme::new(true, false, None));
    let mut t = Interaction::new(Editor::new(128, 1));
    assert!(matches!(
        t.open_with_config(&slave, &slave, Prompt::new("caps").unwrap(), c),
        Err(Error::CapabilityMismatch(_))
    ));
    assert_eq!(format!("{:?}", tcgetattr(&slave).unwrap()), saved);
    assert!(drain(&master).is_empty());
    resize(&slave, 80);
    for property in ["cursor", "erase", "paste"] {
        let mut c = c;
        match property {
            "cursor" => c.facts.cursor = S::Unavailable,
            "erase" => c.facts.erase = S::Unknown,
            _ => {
                c.facts.bracketed_paste = S::Unavailable;
                c.bracketed_paste = P::Required;
            }
        }
        assert!(matches!(
            t.open_with_config(&slave, &slave, Prompt::new("caps").unwrap(), c),
            Err(Error::CapabilityMismatch(_))
        ));
        assert_eq!(format!("{:?}", tcgetattr(&slave).unwrap()), saved);
        assert!(drain(&master).is_empty());
    }
    let redirected = std::fs::File::create("/dev/null").unwrap();
    assert!(matches!(
        t.open_with_config(&slave, &redirected, Prompt::new("caps").unwrap(), c),
        Err(Error::UnsuitableTerminal)
    ));
    assert_eq!(format!("{:?}", tcgetattr(&slave).unwrap()), saved);
    assert!(drain(&master).is_empty());
    // Legacy session shares the resolver and records its compatibility assumption.
    t.open_with_theme(
        &slave,
        &slave,
        Prompt::new("legacy").unwrap(),
        Theme::new(true, false, Some("dumb")),
    )
    .unwrap();
    assert_eq!(t.capabilities().unwrap().requirements, R::Editing);
    assert_eq!(t.capabilities().unwrap().facts.cursor, S::Assumed);
    assert!(!t.features().unwrap().styling);
    resize(&slave, 40);
    t.poll(std::time::Duration::ZERO).unwrap();
    assert_eq!(
        t.capabilities().unwrap().realization.dimensions,
        Some((40, 24))
    );
    t.close().unwrap();
    assert_eq!(format!("{:?}", tcgetattr(&slave).unwrap()), saved);
    eprintln!(
        "NEGATIVE missing dimensions/cursor/erase/required paste: CapabilityMismatch, zero writes, termios exact; legacy TERM=dumb: cursor Assumed, plain, polling resize observed"
    );
}
