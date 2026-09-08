//! Real Linux/macOS PTYs with an independent VT terminal-state oracle.
#![cfg(any(target_os = "linux", target_os = "macos"))]
use replai::{EditError, Editor, Error, Event, Interaction, Prompt, Role};
use rustix::{
    fs::{Mode, OFlags, fcntl_getfl, fcntl_setfl, open},
    io::{read, write},
    termios::{
        InputModes, OptionalActions, SpecialCodeIndex, Winsize, tcgetattr, tcsetattr, tcsetwinsize,
        ttyname,
    },
};
use std::{
    os::fd::OwnedFd,
    sync::{Mutex, MutexGuard},
    time::Duration,
};

#[path = "support/posix_pty.rs"]
mod pty_support;

static SERIAL: Mutex<()> = Mutex::new(());
fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|p| p.into_inner())
}
fn pty(cols: u16, rows: u16) -> (OwnedFd, OwnedFd) {
    let (master, slave) = pty_support::pair();
    resize(&slave, cols, rows);
    fcntl_setfl(&master, fcntl_getfl(&master).unwrap() | OFlags::NONBLOCK).unwrap();
    (master, slave)
}
fn resize(slave: &OwnedFd, cols: u16, rows: u16) {
    tcsetwinsize(
        slave,
        Winsize {
            ws_col: cols,
            ws_row: rows,
            ws_xpixel: 0,
            ws_ypixel: 0,
        },
    )
    .unwrap();
}
fn drain(master: &OwnedFd) -> Vec<u8> {
    let mut out = Vec::new();
    let mut buf = [0; 8192];
    loop {
        match read(master, &mut buf) {
            Ok(0) | Err(rustix::io::Errno::AGAIN | rustix::io::Errno::IO) => return out,
            Ok(n) => out.extend_from_slice(&buf[..n]),
            Err(e) => panic!("PTY read failed: {e}"),
        }
    }
}
fn feed(
    t: &mut Interaction,
    master: &OwnedFd,
    bytes: &[u8],
    screen: &mut vt100::Parser,
) -> Vec<Event> {
    let mut events = Vec::new();
    for b in bytes {
        assert_eq!(write(master, &[*b]).unwrap(), 1);
        if let Some(e) = t.poll(Duration::from_millis(20)).unwrap() {
            events.push(e);
        }
        screen.process(&drain(master));
    }
    events
}
fn termios(slave: &OwnedFd) -> String {
    format!("{:?}", tcgetattr(slave).unwrap())
}
fn prompt() -> Prompt {
    Prompt::new("demo").unwrap()
}

#[test]
fn ready_burst_preserves_host_boundaries_and_typeahead_across_reopen() {
    let _serial = serial();
    let (master, slave) = pty(80, 24);
    let before = termios(&slave);
    let mut t = Interaction::new(Editor::new(4, 2));
    t.open(&slave, &slave, prompt()).unwrap();
    drain(&master);
    let wire = b"ab\tcdef\rnext\r";
    assert_eq!(write(&master, wire).unwrap(), wire.len());
    assert_eq!(
        t.poll(Duration::from_millis(20)).unwrap(),
        Some(Event::CompletionRequested)
    );
    assert_eq!((t.editor().text(), t.editor().cursor()), ("ab", 2));
    t.complete(0..2, "Z").unwrap();
    t.external_output(Role::Dim, "notice").unwrap();
    assert_eq!(
        t.poll(Duration::ZERO).unwrap(),
        Some(Event::Rejected(EditError::Capacity))
    );
    assert_eq!((t.editor().text(), t.editor().cursor()), ("Zcde", 4));
    assert_eq!(
        t.poll(Duration::ZERO).unwrap(),
        Some(Event::Submitted("Zcde".into()))
    );
    assert_eq!(termios(&slave), before);
    assert!(!t.is_open());
    t.editor_mut().unwrap().clear();
    drain(&master);
    t.open(&slave, &slave, prompt()).unwrap();
    assert_eq!(
        t.poll(Duration::ZERO).unwrap(),
        Some(Event::Submitted("next".into()))
    );
    assert_eq!(termios(&slave), before);
    let output = drain(&master);
    assert!(output.windows(4).any(|w| w == b"next"));
}

#[test]
fn ready_interrupt_eof_and_explicit_close_retain_following_input() {
    let _serial = serial();
    let (master, slave) = pty(80, 24);
    let mut t = Interaction::new(Editor::new(100, 1));
    t.open(&slave, &slave, prompt()).unwrap();
    assert_eq!(write(&master, b"first\x03\x04tail\tmore\r").unwrap(), 17);
    assert_eq!(t.poll(Duration::ZERO).unwrap(), Some(Event::Interrupted));
    assert_eq!(t.editor().text(), "first");
    t.editor_mut().unwrap().clear();
    t.open(&slave, &slave, prompt()).unwrap();
    assert_eq!(t.poll(Duration::ZERO).unwrap(), Some(Event::EndOfInput));
    t.open(&slave, &slave, prompt()).unwrap();
    assert_eq!(
        t.poll(Duration::ZERO).unwrap(),
        Some(Event::CompletionRequested)
    );
    assert_eq!(t.editor().text(), "tail");
    t.close().unwrap();
    t.open(&slave, &slave, prompt()).unwrap();
    assert_eq!(
        t.poll(Duration::ZERO).unwrap(),
        Some(Event::Submitted("tailmore".into()))
    );
}

#[test]
fn pty_editing_submission_and_exact_termios_restore() {
    let _serial = serial();
    let (master, slave) = pty(80, 24);
    let mut unusual = tcgetattr(&slave).unwrap();
    unusual.input_modes.insert(InputModes::IXOFF);
    unusual.special_codes[SpecialCodeIndex::VMIN] = 3;
    unusual.special_codes[SpecialCodeIndex::VTIME] = 7;
    tcsetattr(&slave, OptionalActions::Now, &unusual).unwrap();
    let before = termios(&slave);
    let editor = Editor::new(100, 3);
    let mut t = Interaction::new(editor);
    t.open(&slave, &slave, prompt()).unwrap();
    assert_ne!(termios(&slave), before);
    let mut screen = vt100::Parser::new(24, 80, 100);
    let first = drain(&master);
    assert!(first.starts_with(b"\x1b[?2004h\r\x1b[2K"));
    screen.process(&first);
    assert_eq!(screen.screen().contents(), "demo> ");
    assert_eq!(screen.screen().cursor_position(), (0, 6));
    feed(&mut t, &master, "hé界🌍".as_bytes(), &mut screen);
    feed(&mut t, &master, b"\x1b[D\x7fX", &mut screen);
    assert_eq!(t.editor().text(), "héX🌍");
    assert_eq!(screen.screen().contents(), "demo> héX🌍");
    assert_eq!(screen.screen().cursor_position(), (0, 9));
    assert_eq!(
        feed(&mut t, &master, b"\r", &mut screen),
        [Event::Submitted("héX🌍".into())]
    );
    assert_eq!(termios(&slave), before);
    assert!(!screen.screen().bracketed_paste());
}

#[test]
fn pty_history_and_completion_preserve_host_control() {
    let _serial = serial();
    let (master, slave) = pty(80, 24);
    let mut editor = Editor::new(100, 2);
    editor.admit_history("earlier").unwrap();
    editor.insert("draft").unwrap();
    editor.left();
    let mut t = Interaction::new(editor);
    t.open(&slave, &slave, prompt()).unwrap();
    let mut screen = vt100::Parser::new(24, 80, 100);
    screen.process(&drain(&master));
    feed(&mut t, &master, b"\x1b[A\x1b[B", &mut screen);
    assert_eq!((t.editor().text(), t.editor().cursor()), ("draft", 4));
    for answer in [
        Ok(vec![]),
        Ok(vec!["first", "second"]),
        Err("lookup failed"),
    ] {
        assert_eq!(
            feed(&mut t, &master, b"\t", &mut screen),
            [Event::CompletionRequested]
        );
        // The host can return zero/ambiguous candidates or fail independently of
        // the editor. Only a host-selected unique answer becomes a transaction.
        if let Ok(candidates) = answer
            && let [replacement] = candidates.as_slice()
        {
            t.complete(0..t.editor().text().len(), replacement).unwrap();
        }
        assert_eq!(t.editor().text(), "draft");
    }
    assert!(matches!(
        t.complete(0..99, "bad"),
        Err(Error::Edit(EditError::InvalidRange))
    ));
    assert_eq!(t.editor().text(), "draft");
    t.complete(0..5, "café界").unwrap();
    screen.process(&drain(&master));
    assert_eq!(screen.screen().contents(), "demo> café界");
    assert_eq!(screen.screen().cursor_position(), (0, 12));
    assert!(matches!(
        t.complete(4..5, "x"),
        Err(Error::Edit(EditError::InvalidRange))
    ));
    t.close().unwrap();
}

#[test]
fn pty_multiline_paste_resize_clear_and_external_output() {
    let _serial = serial();
    let (master, slave) = pty(12, 12);
    let before = termios(&slave);
    let editor = Editor::new(200, 2);
    let mut t = Interaction::new(editor);
    t.open(&slave, &slave, prompt()).unwrap();
    let mut screen = vt100::Parser::new(12, 12, 100);
    screen.process(&drain(&master));
    assert!(
        feed(
            &mut t,
            &master,
            "\x1b[200~ab界\r\nline 🌍\x1b[201~".as_bytes(),
            &mut screen
        )
        .is_empty()
    );
    assert_eq!(t.editor().text(), "ab界\nline 🌍");
    assert_eq!(screen.screen().contents(), "demo> ab界\n... line 🌍");
    feed(&mut t, &master, b"\x1b[D", &mut screen);
    let cursor = t.editor().cursor();
    t.external_output(Role::Dim, "notice without newline")
        .unwrap();
    screen.process(&drain(&master));
    assert_eq!(t.editor().cursor(), cursor);
    assert!(
        screen
            .screen()
            .contents()
            .contains("demo> ab界\n... line 🌍")
    );
    assert_eq!(screen.screen().cursor_position(), (3, 9));
    resize(&slave, 9, 12);
    screen.screen_mut().set_size(12, 9);
    t.poll(Duration::ZERO).unwrap();
    screen.process(&drain(&master));
    assert_eq!(t.editor().cursor(), cursor);
    feed(&mut t, &master, b"X", &mut screen);
    assert_eq!(t.editor().text(), "ab界\nline X🌍");
    feed(&mut t, &master, b"\x0c", &mut screen);
    assert!(
        screen
            .screen()
            .contents()
            .starts_with("demo> ab\n界\n... line"),
        "{}",
        screen.screen().contents()
    );
    assert_eq!(
        feed(&mut t, &master, b"\r", &mut screen),
        [Event::Submitted("ab界\nline X🌍".into())]
    );
    assert_eq!(termios(&slave), before);
}

#[test]
fn pty_interrupt_eof_and_nonempty_ctrl_d_are_distinct() {
    let _serial = serial();
    let (master, slave) = pty(80, 24);
    let before = termios(&slave);
    for (input, expected) in [
        (b"draft\x03".as_slice(), Event::Interrupted),
        (b"\x04", Event::EndOfInput),
        (b"abc\x04\x01\x04\r", Event::Submitted("bc".into())),
    ] {
        let editor = Editor::new(100, 2);
        let mut t = Interaction::new(editor);
        t.open(&slave, &slave, prompt()).unwrap();
        let mut screen = vt100::Parser::new(24, 80, 10);
        screen.process(&drain(&master));
        let interrupted = expected == Event::Interrupted;
        assert_eq!(feed(&mut t, &master, input, &mut screen), [expected]);
        if interrupted {
            assert!(screen.screen().contents().contains("^C"));
        }
        assert_eq!(termios(&slave), before);
        assert!(!screen.screen().bracketed_paste());
    }
}

#[test]
fn pty_rejections_leave_valid_editable_drafts() {
    let _serial = serial();
    let (master, slave) = pty(80, 24);
    let editor = Editor::new(5, 1);
    let mut t = Interaction::new(editor);
    t.open(&slave, &slave, prompt()).unwrap();
    let mut screen = vt100::Parser::new(24, 80, 10);
    screen.process(&drain(&master));
    feed(&mut t, &master, b"ok", &mut screen);
    for (bytes, error) in [
        (
            b"\x1b[200~123456789\x1b[201~".as_slice(),
            EditError::Capacity,
        ),
        (b"\xff", EditError::InvalidUtf8),
        (b"\x1b[200~\x03\x1b[201~", EditError::InvalidText),
        (b"\x1b[999~", EditError::InvalidSequence),
    ] {
        assert_eq!(
            feed(&mut t, &master, bytes, &mut screen),
            [Event::Rejected(error)]
        );
        assert_eq!(t.editor().text(), "ok");
    }
    feed(&mut t, &master, b"123", &mut screen);
    assert_eq!(
        feed(&mut t, &master, b"x", &mut screen),
        [Event::Rejected(EditError::Capacity)]
    );
    assert_eq!(t.editor().text(), "ok123");
    t.close().unwrap();
}

#[test]
fn pty_partial_paste_times_out_with_restoration() {
    let _serial = serial();
    let (master, slave) = pty(80, 24);
    let before = termios(&slave);
    let editor = Editor::new(100, 1);
    let mut t = Interaction::new(editor);
    t.open(&slave, &slave, prompt()).unwrap();
    let mut screen = vt100::Parser::new(24, 80, 10);
    screen.process(&drain(&master));
    feed(
        &mut t,
        &master,
        b"draft\x1b[200~partial\x1b[20",
        &mut screen,
    );
    let mut failed = false;
    for _ in 0..5 {
        if t.poll(Duration::from_millis(100)).is_err() {
            failed = true;
            break;
        }
    }
    assert!(failed);
    assert_eq!(t.editor().text(), "draft");
    assert_eq!(termios(&slave), before);
    screen.process(&drain(&master));
    assert!(!screen.screen().bracketed_paste());
}

#[test]
fn pty_partial_acquisition_and_read_failure_restore_captured_state() {
    let _serial = serial();
    let (master, slave) = pty(80, 24);
    let before = termios(&slave);
    let name = ttyname(&slave, Vec::new()).unwrap();
    let readonly = open(&name, OFlags::RDONLY | OFlags::NOCTTY, Mode::empty()).unwrap();
    let writeonly = open(&name, OFlags::WRONLY | OFlags::NOCTTY, Mode::empty()).unwrap();
    let editor = Editor::new(100, 1);
    let mut t = Interaction::new(editor);
    assert!(t.open(&slave, &readonly, prompt()).is_err());
    assert_eq!(termios(&slave), before);
    t.open(&writeonly, &slave, prompt()).unwrap();
    drain(&master);
    write(&master, b"x").unwrap();
    assert!(matches!(
        t.poll(Duration::from_millis(20)),
        Err(Error::Io(_))
    ));
    assert_eq!(termios(&slave), before);
    assert!(drain(&master).windows(8).any(|w| w == b"\x1b[?2004l"));
}

#[test]
fn pty_unwinding_close_and_exclusivity() {
    let _serial = serial();
    let (master, slave) = pty(80, 24);
    let before = termios(&slave);
    #[cfg(target_os = "linux")]
    let signals = || {
        std::fs::read_to_string("/proc/self/status")
            .unwrap()
            .lines()
            .filter(|l| {
                l.starts_with("SigBlk:") || l.starts_with("SigCgt:") || l.starts_with("SigIgn:")
            })
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    #[cfg(target_os = "linux")]
    let prior_signals = signals();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let editor = Editor::new(100, 1);
        let mut t = Interaction::new(editor);
        t.open(&slave, &slave, prompt()).unwrap();
        let mut other = Interaction::new(Editor::new(100, 1));
        assert!(other.open(&slave, &slave, prompt()).is_err());
        panic!("ordinary host unwinding");
    }));
    assert!(result.is_err());
    assert_eq!(termios(&slave), before);
    #[cfg(target_os = "linux")]
    assert_eq!(signals(), prior_signals);
    let bytes = drain(&master);
    assert!(bytes.windows(8).any(|w| w == b"\x1b[?2004l"));
    let editor = Editor::new(100, 1);
    let mut t = Interaction::new(editor);
    t.open(&slave, &slave, prompt()).unwrap();
    t.close().unwrap();
    t.close().unwrap();
    assert_eq!(termios(&slave), before);
}

#[test]
fn non_tty_and_mismatched_terminals_fail_before_mutation() {
    let _serial = serial();
    let (master, slave) = pty(80, 24);
    let (_m2, s2) = pty(80, 24);
    let before = termios(&slave);
    let null = std::fs::File::open("/dev/null").unwrap();
    let editor = Editor::new(100, 1);
    let mut t = Interaction::new(editor);
    assert!(t.open(&null, &slave, prompt()).is_err());
    assert!(t.open(&slave, &null, prompt()).is_err());
    assert!(t.open(&slave, &s2, prompt()).is_err());
    resize(&slave, 1, 24);
    assert!(t.open(&slave, &slave, prompt()).is_err());
    assert_eq!(termios(&slave), before);
    assert!(drain(&master).is_empty());
}

// A separate process gives color environment tests isolation without mutating
// process-global environment while other Rust tests are running.
#[test]
fn pty_reference_child() {
    use std::io::Write;
    let Ok(_case) = std::env::var("REPLAI_TEST_REFERENCE") else {
        return;
    };
    let mut editor = Editor::new(1024, 2);
    editor.admit_history("earlier").unwrap();
    let mut t = Interaction::new(editor);
    t.open(&std::io::stdin(), &std::io::stdout(), prompt())
        .unwrap();
    std::io::stderr().write_all(b"O").unwrap();
    loop {
        match t.poll(Duration::from_millis(20)).unwrap() {
            Some(Event::CompletionRequested) => std::io::stderr().write_all(b"R").unwrap(),
            Some(Event::Interrupted) => {
                std::io::stderr().write_all(b"R").unwrap();
                break;
            }
            Some(Event::Submitted(_) | Event::EndOfInput) => break,
            _ => {}
        }
    }
}

struct Child(std::process::Child);
impl Drop for Child {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn acknowledge(child: &mut Child, expected: u8) {
    let stderr = child.0.stderr.as_ref().unwrap();
    let mut fds = [rustix::event::PollFd::new(
        stderr,
        rustix::event::PollFlags::IN,
    )];
    let deadline = rustix::event::Timespec {
        tv_sec: 5,
        tv_nsec: 0,
    };
    assert!(rustix::event::poll(&mut fds, Some(&deadline)).unwrap() > 0);
    let mut byte = [0];
    assert_eq!(read(stderr, &mut byte).unwrap(), 1);
    assert_eq!(byte, [expected]);
}
fn assert_screen(actual: &vt100::Screen, expected: &vt100::Screen, rows: u16, cols: u16) {
    assert_eq!(actual.contents(), expected.contents());
    assert_eq!(actual.cursor_position(), expected.cursor_position());
    for row in 0..rows {
        for col in 0..cols {
            let a = actual.cell(row, col).unwrap();
            let e = expected.cell(row, col).unwrap();
            assert_eq!((a.fgcolor(), a.bold()), (e.fgcolor(), e.bold()));
            assert_eq!(a.bgcolor(), vt100::Color::Default);
        }
    }
    assert!(!actual.alternate_screen());
}

#[test]
fn pty_reference_records_match_text_cursor_color_and_native_background() {
    let _serial = serial();
    for record in include_str!("fixtures/presentation.tsv").lines() {
        let (case, hex) = record.split_once('\t').unwrap();
        let reference: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let (master, slave) = pty(80, 24);
        let before = termios(&slave);
        let mut command = std::process::Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "pty_reference_child",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("REPLAI_TEST_REFERENCE", case)
            .env_remove("NO_COLOR")
            .env(
                "TERM",
                if case == "dumb" {
                    "dumb"
                } else {
                    "xterm-256color"
                },
            )
            .stdin(std::process::Stdio::from(rustix::io::dup(&slave).unwrap()))
            .stdout(std::process::Stdio::from(rustix::io::dup(&slave).unwrap()))
            .stderr(std::process::Stdio::piped());
        if case == "no_color" {
            command.env("NO_COLOR", "");
        }
        let mut child = Child(command.spawn().unwrap());
        acknowledge(&mut child, b'O');
        let initial = drain(&master);
        let start = initial
            .windows(8)
            .position(|w| w == b"\x1b[?2004h")
            .unwrap();
        let mut capture = initial[start + 8..].to_vec();
        if matches!(case, "styled_prompt" | "no_color" | "dumb") {
            assert_eq!(capture, reference, "initial prompt bytes: {case}");
        }
        let input: &[u8] = match case {
            "typed" => b"hello\t",
            "left" => b"hello\x1b[D\t",
            "history" => b"\x1b[A\t",
            "paste" => b"\x1b[200~hello\r\nworld\x1b[201~\t",
            "clear" => b"hello\x0c\t",
            "interrupt" => b"hello\x03",
            "resize" => b"draft\t",
            _ => b"\t",
        };
        assert_eq!(write(&master, input).unwrap(), input.len());
        acknowledge(&mut child, b'R');
        capture.extend(drain(&master));
        let cols = if case == "resize" { 60 } else { 80 };
        if case == "resize" {
            resize(&slave, cols, 24);
            write(&master, b"\t").unwrap();
            acknowledge(&mut child, b'R');
            capture.extend(drain(&master));
        }
        // The harness may print after the child closes; exclude only its output.
        if let Some(end) = capture.windows(8).position(|w| w == b"\x1b[?2004l") {
            capture.truncate(end);
        }
        let mut expected = vt100::Parser::new(24, cols, 100);
        expected.process(&reference);
        let mut actual = vt100::Parser::new(24, cols, 100);
        actual.process(&capture);
        assert_screen(actual.screen(), expected.screen(), 24, cols);
        if case != "interrupt" {
            write(&master, b"\r").unwrap();
        }
        assert!(child.0.wait().unwrap().success());
        assert_eq!(termios(&slave), before, "cleanup: {case}");
    }
}

#[test]
fn pty_fragmented_sequences_survive_delays_beyond_a_per_byte_guess() {
    let _serial = serial();
    let (master, slave) = pty(80, 24);
    let editor = Editor::new(100, 0);
    let mut t = Interaction::new(editor);
    t.open(&slave, &slave, prompt()).unwrap();
    let mut screen = vt100::Parser::new(24, 80, 0);
    screen.process(&drain(&master));
    for &byte in "\x1b[200~e\u{301}界\x1b[201~\x1b[D".as_bytes() {
        assert!(feed(&mut t, &master, &[byte], &mut screen).is_empty());
        assert!(t.poll(Duration::from_millis(35)).unwrap().is_none());
    }
    assert_eq!((t.editor().text(), t.editor().cursor()), ("e\u{301}界", 3));
    assert_eq!(screen.screen().cursor_position(), (0, 7));
    t.close().unwrap();
}

#[test]
fn pty_wrap_boundaries_and_tall_drafts_keep_the_cursor_on_the_edit() {
    let _serial = serial();
    let (master, slave) = pty(8, 6);
    let editor = Editor::new(100, 0);
    let mut t = Interaction::new(editor);
    t.open(&slave, &slave, prompt()).unwrap();
    let mut screen = vt100::Parser::new(6, 8, 100);
    screen.process(&drain(&master));
    feed(&mut t, &master, "a界".as_bytes(), &mut screen);
    feed(&mut t, &master, b"\x1b[D", &mut screen);
    assert_eq!(screen.screen().cursor_position(), (1, 0));
    assert_eq!(screen.screen().contents(), "demo> a\n界");
    t.complete(0..4, "ab\nc").unwrap();
    screen.process(&drain(&master));
    assert_eq!(screen.screen().contents(), "demo> ab\n... c");
    assert_eq!(screen.screen().cursor_position(), (1, 5));
    t.complete(0..4, "1\n2\n3\n4\n5\n6\n7").unwrap();
    screen.process(&drain(&master));
    assert!(screen.screen().contents().ends_with("... 7"));
    feed(&mut t, &master, b"\x01", &mut screen);
    assert!(screen.screen().contents().starts_with("demo> 1"));
    assert_eq!(screen.screen().cursor_position(), (0, 6));
    resize(&slave, 10, 4);
    screen.screen_mut().set_size(4, 10);
    t.poll(Duration::ZERO).unwrap();
    screen.process(&drain(&master));
    assert!(screen.screen().contents().starts_with("demo> 1"));
    assert_eq!(screen.screen().cursor_position(), (0, 6));
    feed(&mut t, &master, b"\x05", &mut screen);
    assert!(screen.screen().contents().ends_with("... 7"));
    assert_eq!(t.editor().text(), "1\n2\n3\n4\n5\n6\n7");
    t.close().unwrap();
}

#[test]
fn pty_external_output_rejects_controls_and_keeps_queued_input() {
    let _serial = serial();
    let (master, slave) = pty(80, 24);
    let before = termios(&slave);
    let mut editor = Editor::new(100, 0);
    editor.insert("ab").unwrap();
    editor.left();
    let mut t = Interaction::new(editor);
    t.open(&slave, &slave, prompt()).unwrap();
    let mut screen = vt100::Parser::new(24, 80, 0);
    screen.process(&drain(&master));
    assert!(matches!(
        t.external_output(Role::Error, "bad\x1b[2J"),
        Err(Error::Edit(EditError::InvalidText))
    ));
    assert!(drain(&master).is_empty());
    assert_eq!((t.editor().text(), t.editor().cursor()), ("ab", 1));
    write(&master, b"X").unwrap();
    t.external_output(Role::Dim, "notice\r\n").unwrap();
    screen.process(&drain(&master));
    assert_eq!(screen.screen().contents(), "notice\ndemo> ab");
    assert_eq!(screen.screen().cursor_position(), (1, 7));
    t.poll(Duration::ZERO).unwrap();
    screen.process(&drain(&master));
    assert_eq!(t.editor().text(), "aXb");
    assert_eq!(screen.screen().cursor_position(), (1, 8));
    assert_eq!(t.interrupt().unwrap(), Event::Interrupted);
    assert_eq!(termios(&slave), before);
}

#[cfg(target_os = "macos")]
#[test]
fn darwin_restores_all_settled_attributes_without_consuming_queued_input() {
    let _serial = serial();
    let (master, slave) = pty(80, 24);
    // FIONREAD resolves Darwin's transient PENDIN state; no fields are masked.
    rustix::io::ioctl_fionread(&slave).unwrap();
    let before = termios(&slave);
    let mut t = Interaction::new(Editor::new(100, 2));
    t.open(&slave, &slave, prompt()).unwrap();
    drain(&master);
    write(&master, b"next\n").unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(1);
    while rustix::io::ioctl_fionread(&slave).unwrap() != 5 {
        assert!(
            std::time::Instant::now() < deadline,
            "PTY input did not become ready"
        );
        std::thread::yield_now();
    }
    t.close().unwrap();
    rustix::io::ioctl_fionread(&slave).unwrap();
    assert_eq!(termios(&slave), before);
    // Bytes queued while raw need not already form a Darwin canonical record.
    // Reopen and prove exact delivery, rather than assuming canonical readiness.
    t.open(&slave, &slave, prompt()).unwrap();
    let mut submitted = None;
    for _ in 0..10 {
        if let Some(event) = t.poll(Duration::from_millis(10)).unwrap() {
            submitted = Some(event);
            break;
        }
    }
    assert_eq!(
        submitted,
        Some(Event::Submitted("next".into())),
        "draft={:?}, queued={}, before={}",
        t.editor().text(),
        rustix::io::ioctl_fionread(&slave).unwrap(),
        before
    );
    rustix::io::ioctl_fionread(&slave).unwrap();
    assert_eq!(termios(&slave), before);
}

#[test]
fn pty_combining_cjk_joined_emoji_variation_and_regional_edits_are_atomic() {
    let _serial = serial();
    for grapheme in ["e\u{301}", "界", "👩\u{200d}💻", "🇮🇹", "♥\u{fe0f}"] {
        let (master, slave) = pty(80, 24);
        let before = termios(&slave);
        let mut t = Interaction::new(Editor::new(1024, 2));
        t.open(&slave, &slave, prompt()).unwrap();
        let mut screen = vt100::Parser::new(24, 80, 100);
        screen.process(&drain(&master));
        feed(
            &mut t,
            &master,
            format!("A{grapheme}B").as_bytes(),
            &mut screen,
        );
        assert_eq!(t.editor().text(), format!("A{grapheme}B"));
        feed(&mut t, &master, b"\x1b[D\x7f", &mut screen);
        assert_eq!((t.editor().text(), t.editor().cursor()), ("AB", 1));
        assert_eq!(screen.screen().contents(), "demo> AB");
        assert_eq!(screen.screen().cursor_position(), (0, 7));
        t.close().unwrap();
        assert_eq!(termios(&slave), before);
    }
}

#[test]
fn structured_output_preserves_draft_cursor_and_posix_lifecycle() {
    use replai::{Alignment, Block, Column, Document, Severity, Span, Text, Theme};
    let _serial = serial();
    for columns in [8, 60] {
        for theme in [
            Theme::new(true, false, None),
            Theme::new(true, true, None),
            Theme::new(true, false, Some("dumb")),
        ] {
            let (master, slave) = pty(columns, 30);
            let before = termios(&slave);
            let mut editor = Editor::new(1024, 2);
            editor.insert("a界b\nc").unwrap();
            editor.left();
            let prompt = Prompt::composed(
                Text::from_spans(vec![
                    Span::new(Role::Strong, "db").unwrap(),
                    Span::new(Role::Accent, "> ").unwrap(),
                ])
                .unwrap(),
            )
            .unwrap()
            .with_continuation_text(Text::styled(Role::Dim, ".. ").unwrap())
            .unwrap();
            let mut interaction = Interaction::new(editor);
            interaction
                .open_with_theme(&slave, &slave, prompt, theme)
                .unwrap();
            let initial = drain(&master);
            let mut screen = vt100::Parser::new(30, columns, 0);
            screen.process(&initial);
            let t = |s: &str| Text::new(s).unwrap();
            let doc = Document::new(vec![
                Block::Heading {
                    level: 2,
                    text: t("Facts"),
                },
                Block::Table {
                    columns: vec![
                        Column {
                            heading: t("Key"),
                            alignment: Alignment::Left,
                        },
                        Column {
                            heading: t("Value"),
                            alignment: Alignment::Left,
                        },
                    ],
                    rows: vec![vec![t("Name"), t("café 界")]],
                },
                Block::Status {
                    severity: Severity::Success,
                    text: t("ready"),
                },
            ])
            .unwrap();
            interaction.output_document(&doc).unwrap();
            let bytes = drain(&master);
            screen.process(&bytes);
            let mut expected = vt100::Parser::new(30, columns, 0);
            expected.process(
                doc.render(columns as usize, theme)
                    .unwrap()
                    .replace('\n', "\r\n")
                    .as_bytes(),
            );
            // Independent document screen plus original editing frame is the oracle.
            expected.process(&initial);
            assert_eq!(screen.screen().contents(), expected.screen().contents());
            assert_eq!(
                screen.screen().cursor_position(),
                expected.screen().cursor_position()
            );
            assert_eq!(
                (interaction.editor().text(), interaction.editor().cursor()),
                ("a界b\nc", 6)
            );
            for row in 0..30 {
                for col in 0..columns {
                    let actual = screen.screen().cell(row, col).unwrap();
                    let reference = expected.screen().cell(row, col).unwrap();
                    assert_eq!(actual.fgcolor(), reference.fgcolor());
                    assert_eq!(actual.bold(), reference.bold());
                    assert_eq!(
                        screen.screen().cell(row, col).unwrap().bgcolor(),
                        vt100::Color::Default
                    );
                }
            }
            resize(&slave, columns + 3, 30);
            screen.screen_mut().set_size(30, columns + 3);
            interaction.poll(Duration::from_millis(20)).unwrap();
            let resized = drain(&master);
            screen.process(&resized);
            interaction.output_document(&doc).unwrap();
            let more = drain(&master);
            screen.process(&more);
            assert!(more.windows(5).any(|w| w == b"ready"));
            let (row, col) = screen.screen().cursor_position();
            assert_eq!(col, 3); // continuation, cursor before c
            assert!(screen.screen().contents().ends_with(".. c"));
            assert_eq!(screen.screen().cell(row, col).unwrap().contents(), "c");
            assert_eq!(
                (interaction.editor().text(), interaction.editor().cursor()),
                ("a界b\nc", 6)
            );
            interaction.close().unwrap();
            let close = drain(&master);
            assert_eq!(termios(&slave), before);
            let all = [initial, bytes, resized, more, close].concat();
            assert_eq!(
                all.windows(8).filter(|w| *w == b"\x1b[?2004h").count(),
                all.windows(8).filter(|w| *w == b"\x1b[?2004l").count()
            );
        }
    }
}
