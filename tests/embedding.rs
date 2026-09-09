//! Public POSIX admission, caller ownership and unwinding across drive states.
#![cfg(any(target_os = "linux", target_os = "macos"))]
use replai::{
    Editor, Error, FeaturePolicy, FeatureSupport, Interaction, Prompt, TerminalConfig,
    TerminalFacts, Theme, WaitInterest, Wake,
};
use rustix::{
    fs::{OFlags, fcntl_getfl, fcntl_setfl},
    io::{read, write},
    termios::{Winsize, tcgetattr, tcsetwinsize},
};
#[path = "support/posix_pty.rs"]
mod pty_support;

#[test]
fn admission_borrowing_and_unwinding_preserve_real_caller_resources() {
    let (master, slave) = pty_support::pair();
    tcsetwinsize(
        &slave,
        Winsize {
            ws_col: 80,
            ws_row: 24,
            ws_xpixel: 0,
            ws_ypixel: 0,
        },
    )
    .unwrap();
    fcntl_setfl(&master, fcntl_getfl(&master).unwrap() | OFlags::NONBLOCK).unwrap();
    let before = format!("{:?}", tcgetattr(&slave).unwrap());
    let flags = fcntl_getfl(&slave).unwrap();
    let config = TerminalConfig {
        facts: TerminalFacts::assumed_vt(),
        styling: FeaturePolicy::Disabled,
        bracketed_paste: FeaturePolicy::Preferred,
        theme: Theme::new(true, true, None),
    };
    let mut interaction = Interaction::new(Editor::new(1024, 4));
    let mut unavailable = config;
    unavailable.facts.cursor = FeatureSupport::Unknown;
    assert!(matches!(
        interaction.open_with_config(&slave, &slave, Prompt::new("test").unwrap(), unavailable),
        Err(Error::CapabilityMismatch(_))
    ));
    assert_eq!(format!("{:?}", tcgetattr(&slave).unwrap()), before);
    assert!(!interaction.is_open());
    for wire in [
        b"".as_slice(),
        b"abc\tremaining",
        b"\x1b[",
        b"\x1b[200~pending",
    ] {
        for unwind in [false, true] {
            let mut t = Interaction::new(Editor::new(1024, 4));
            t.open_with_config(&slave, &slave, Prompt::new("test").unwrap(), config)
                .unwrap();
            let source = t.input_source().unwrap();
            assert_eq!(fcntl_getfl(source).unwrap(), flags);
            assert_eq!(
                fcntl_getfl(&slave).unwrap(),
                flags,
                "no caller O_NONBLOCK mutation"
            );
            if !wire.is_empty() {
                write(&master, wire).unwrap();
                t.advance(Wake::InputReady).unwrap();
            }
            if wire == b"abc\tremaining" {
                assert_eq!(t.wait_interest().unwrap(), WaitInterest::Ready);
            }
            if wire.starts_with(b"\x1b[") {
                assert!(matches!(
                    t.wait_interest().unwrap(),
                    WaitInterest::Input { deadline: Some(_) }
                ));
            }
            if unwind {
                assert!(
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                        let _owned = t;
                        panic!("host unwinds with pending interaction state");
                    }))
                    .is_err()
                );
            } else {
                drop(t);
            }
            assert_eq!(format!("{:?}", tcgetattr(&slave).unwrap()), before);
            assert_eq!(fcntl_getfl(&slave).unwrap(), flags);
            let mut bytes = Vec::new();
            let mut buffer = [0; 4096];
            while let Ok(n) = read(&master, &mut buffer) {
                if n == 0 {
                    break;
                }
                bytes.extend_from_slice(&buffer[..n]);
            }
            let mut screen = vt100::Parser::new(24, 80, 100);
            screen.process(&bytes);
            assert!(!screen.screen().bracketed_paste());
        }
    }
    // Optional paste loss changes admitted modes, never which engine runs.
    let mut config = config;
    config.facts.bracketed_paste = FeatureSupport::Unavailable;
    interaction
        .open_with_config(&slave, &slave, Prompt::new("test").unwrap(), config)
        .unwrap();
    assert!(!interaction.features().unwrap().bracketed_paste);
    interaction.close().unwrap();
    assert_eq!(format!("{:?}", tcgetattr(&slave).unwrap()), before);
}
