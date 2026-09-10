//! Shared engine/transport qualification, run on every native CI platform.
//! The virtual transport owns no FDs and does not masquerade as a system TTY.
use crate::{
    Editor, Error, Event, Prompt, Role, Theme,
    actions::{EditCommand as E, Input, Request as R},
    engine::Engine,
    render::Mutation,
    substrate::{Read, Transport},
    terminal::Terminal,
};
use std::{cell::RefCell, collections::VecDeque, io, rc::Rc, time::Duration};

#[derive(Default)]
struct Device {
    input: VecDeque<u8>,
    output: Vec<u8>,
    size: (usize, usize),
    restored: usize,
    eof: bool,
    fail_read: bool,
    fail_write: bool,
    fail_restore: usize,
    read_limit: usize,
    reads: usize,
    dimension_queries: usize,
    writes: usize,
    waits: Vec<Duration>,
}
#[derive(Clone)]
struct Virtual(Rc<RefCell<Device>>);
impl Virtual {
    fn new() -> Self {
        Self(Rc::new(RefCell::new(Device {
            size: (80, 24),
            read_limit: 4096,
            ..Device::default()
        })))
    }
    fn feed(&self, bytes: &[u8]) {
        self.0.borrow_mut().input.extend(bytes);
    }
    fn screen(&self) -> vt100::Parser {
        let d = self.0.borrow();
        let mut p = vt100::Parser::new(d.size.1 as u16, d.size.0 as u16, 100);
        p.process(&d.output);
        p
    }
}
impl Transport for Virtual {
    fn dimensions(&self) -> io::Result<(usize, usize)> {
        let mut d = self.0.borrow_mut();
        d.dimension_queries += 1;
        Ok(d.size)
    }
    fn read(&self, buffer: &mut [u8], timeout: Duration) -> io::Result<Read> {
        let mut d = self.0.borrow_mut();
        d.reads += 1;
        d.waits.push(timeout);
        if d.fail_read {
            return Err(io::Error::other("read failure"));
        }
        let n = buffer.len().min(d.input.len()).min(d.read_limit);
        if n == 0 {
            return Ok(if d.eof { Read::Eof } else { Read::Idle });
        }
        for slot in &mut buffer[..n] {
            *slot = d.input.pop_front().unwrap();
        }
        Ok(Read::Bytes(n))
    }
    fn write(&self, bytes: &[u8]) -> io::Result<()> {
        let mut d = self.0.borrow_mut();
        if !bytes.is_empty() {
            d.writes += 1;
        }
        if d.fail_write && !bytes.is_empty() {
            return Err(io::Error::other("write failure"));
        }
        d.output.extend_from_slice(bytes);
        Ok(())
    }
    fn cleanup_write(&self, bytes: &[u8]) -> io::Result<()> {
        self.write(bytes)
    }
    fn restore(&mut self) -> io::Result<()> {
        let mut d = self.0.borrow_mut();
        if d.fail_restore > 0 {
            d.fail_restore -= 1;
            return Err(io::Error::other("restore failure"));
        }
        d.restored += 1;
        Ok(())
    }
}

#[test]
fn ready_input_is_coalesced_without_collection_wait_or_event_reordering() {
    for limit in [1, 2, 3, 7, 4096] {
        let d = Virtual::new();
        d.0.borrow_mut().read_limit = limit;
        let (mut e, mut t) = start(&d);
        d.feed("e\u{301}界\x1b[D!\tremaining\r".as_bytes());
        let mut event = t.poll(&mut e, Duration::from_millis(50)).unwrap();
        for _ in 0..100 {
            if event.is_some() {
                break;
            }
            event = t.poll(&mut e, Duration::ZERO).unwrap();
        }
        assert_eq!(event, Some(Event::CompletionRequested));
        assert_eq!((e.editor.text(), e.editor.cursor()), ("e\u{301}!界", 4));
        assert_eq!(d.screen().screen().contents(), "demo> e\u{301}!界");
        if limit == 4096 {
            assert_eq!(d.0.borrow().writes, 2, "initial frame + one ready burst");
        }
        assert!(d.0.borrow().waits.iter().skip(1).all(Duration::is_zero));
        let fx = e.complete(0..4, "done").unwrap();
        t.apply(&mut e, fx).unwrap();
        let mut event = None;
        for _ in 0..100 {
            event = t.poll(&mut e, Duration::ZERO).unwrap();
            if event.is_some() {
                break;
            }
        }
        assert_eq!(event, Some(Event::Submitted("doneremaining界".into())));
        assert!(!t.active);
    }
}

#[test]
fn isolated_key_is_visible_without_probing_another_read() {
    let d = Virtual::new();
    let (mut e, mut t) = start(&d);
    d.feed(b"a");
    assert_eq!(t.poll(&mut e, Duration::from_millis(100)).unwrap(), None);
    assert_eq!(d.0.borrow().reads, 1);
    assert_eq!(d.0.borrow().writes, 2);
    assert_eq!(d.screen().screen().contents(), "demo> a");
}

#[test]
fn buffered_partial_sequences_expire_and_paste_remains_atomic() {
    let d = Virtual::new();
    let (mut e, mut t) = start(&d);
    d.feed(b"ab\x1b[");
    assert_eq!(t.poll(&mut e, Duration::ZERO).unwrap(), None);
    assert_eq!(e.editor.text(), "ab");
    d.feed(b"D!");
    t.poll(&mut e, Duration::ZERO).unwrap();
    assert_eq!(e.editor.text(), "a!b");
    d.feed(b"\x1b[200~x\r\ny\t");
    t.poll(&mut e, Duration::ZERO).unwrap();
    assert_eq!(e.editor.text(), "a!b");
    d.feed(b"\x1b[201~");
    t.poll(&mut e, Duration::ZERO).unwrap();
    assert_eq!(e.editor.text(), "a!x\ny\tb");
    d.feed(b"\x1b[");
    t.poll(&mut e, Duration::ZERO).unwrap();
    t.expire_for_test();
    assert_eq!(
        t.poll(&mut e, Duration::ZERO).unwrap(),
        Some(Event::Rejected(crate::EditError::InvalidSequence))
    );
    assert!(t.active);
    d.feed(b"\x1b[200~unfinished");
    t.poll(&mut e, Duration::ZERO).unwrap();
    t.expire_for_test();
    assert!(t.poll(&mut e, Duration::ZERO).is_err());
    assert!(!t.active);
}

#[test]
fn every_ready_work_budget_flushes_a_correct_bounded_surface() {
    let d = Virtual::new();
    let mut e = Engine::new(Editor::new(100_000, 0));
    let mut t = Terminal::start(
        d.clone(),
        &mut e,
        Prompt::new("demo").unwrap(),
        Theme::new(false, false, None),
    )
    .unwrap();
    let body = "a".repeat(70_000);
    d.feed(format!("\x1b[200~{body}\x1b[201~\r").as_bytes());
    assert_eq!(t.poll(&mut e, Duration::ZERO).unwrap(), None);
    assert!(e.editor.text().is_empty());
    assert!(d.0.borrow().reads <= 8);
    assert_eq!(d.0.borrow().writes, 1);
    let mut outcome = None;
    for _ in 0..10 {
        outcome = t.poll(&mut e, Duration::ZERO).unwrap();
        if outcome.is_some() {
            break;
        }
    }
    assert_eq!(outcome, Some(Event::Submitted(body)));
    assert!(t.pending.len() <= 4096);
}
fn start(device: &Virtual) -> (Engine, Terminal<Virtual>) {
    let mut e = Engine::new(Editor::new(100, 4));
    let t = Terminal::start(
        device.clone(),
        &mut e,
        Prompt::new("demo").unwrap(),
        Theme::new(true, false, None),
    )
    .unwrap();
    (e, t)
}
fn feed(
    device: &Virtual,
    t: &mut Terminal<Virtual>,
    e: &mut Engine,
    bytes: &[u8],
) -> Option<Event> {
    device.feed(bytes);
    let mut event = None;
    for _ in bytes {
        if let Some(value) = t.poll(e, Duration::ZERO).unwrap() {
            event = Some(value);
            break;
        }
        if device.0.borrow().input.is_empty() && t.pending.is_empty() {
            break;
        }
    }
    event
}

#[test]
fn structured_input_and_vt_driver_reach_the_same_engine_and_surface() {
    let device = Virtual::new();
    let (mut driven, mut terminal) = start(&device);
    let mut direct = Engine::new(Editor::new(100, 4));
    let initial = direct
        .start(Prompt::new("demo").unwrap(), (80, 24))
        .unwrap();
    // The engine produces semantic style/text and relative operations, not escapes.
    assert!(
        initial
            .mutations
            .iter()
            .any(|m| matches!(m, Mutation::Style(Role::Accent)))
    );
    let inputs = [
        Input::Text("e\u{301}界🌍".into()),
        Input::Edit(E::Left),
        Input::Text("!".into()),
    ];
    for input in inputs {
        direct.apply(input).unwrap();
    }
    feed(
        &device,
        &mut terminal,
        &mut driven,
        "e\u{301}界🌍\x1b[D!".as_bytes(),
    );
    assert_eq!(
        (direct.editor.text(), direct.editor.cursor()),
        (driven.editor.text(), driven.editor.cursor())
    );
    assert_eq!(direct.editor.text(), "e\u{301}界!🌍");
    assert_eq!(device.screen().screen().cursor_position(), (0, 10));
    assert_eq!(device.screen().screen().contents(), "demo> e\u{301}界!🌍");
    let done = direct.apply(Input::Request(R::Submit)).unwrap();
    assert_eq!(done.event, feed(&device, &mut terminal, &mut driven, b"\r"));
    assert!(!direct.is_open() && !driven.is_open());
    assert_eq!(device.0.borrow().restored, 1);
}

#[test]
fn deterministic_lifecycle_history_completion_output_resize_and_rejection() {
    let mut engine = Engine::new(Editor::new(64, 3));
    engine.editor.admit_history("older").unwrap();
    engine.editor.insert("draft界").unwrap();
    engine.editor.left();
    let original = (engine.editor.text().to_owned(), engine.editor.cursor());
    engine.start(Prompt::new("demo").unwrap(), (20, 5)).unwrap();
    assert!(matches!(
        engine.start(Prompt::new("other").unwrap(), (20, 5)),
        Err(Error::State)
    ));
    engine.apply(Input::Edit(E::HistoryPrevious)).unwrap();
    engine.apply(Input::Edit(E::HistoryNext)).unwrap();
    assert_eq!(
        (engine.editor.text().to_owned(), engine.editor.cursor()),
        original
    );
    assert_eq!(
        engine.apply(Input::Request(R::Completion)).unwrap().event,
        Some(Event::CompletionRequested)
    );
    assert!(engine.complete(6..7, "bad").is_err());
    for bad in ["\x1b[2J", "\x1b]52;c;data\x07", "\x1bPdata\x1b\\", "bad\r"] {
        assert!(engine.external_output(Role::Dim, bad).is_err());
    }
    assert_eq!(
        (engine.editor.text().to_owned(), engine.editor.cursor()),
        original
    );
    let rejection = engine.apply(Input::Text("x".repeat(100))).unwrap();
    assert!(matches!(
        rejection.event,
        Some(Event::Rejected(crate::EditError::Capacity))
    ));
    assert!(rejection.mutations.is_empty());
    engine.complete(0..5, "hello").unwrap();
    let output = engine.external_output(Role::Dim, "notice\r\nnext").unwrap();
    assert!(output.mutations.contains(&Mutation::Paste(false)));
    assert!(output.mutations.contains(&Mutation::Paste(true)));
    assert!(
        output
            .mutations
            .iter()
            .all(|m| !matches!(m, Mutation::Text(t) if t.contains('\x1b') || t.contains('\r')))
    );
    engine.apply(Input::Resize(9, 3)).unwrap();
    assert_eq!(
        (engine.editor.text(), engine.editor.cursor()),
        ("hello界", 5)
    );
    assert!(matches!(
        engine.apply(Input::Resize(1, 1)),
        Err(Error::UnsuitableTerminal)
    ));
    assert!(engine.is_open());
    assert!(
        engine
            .apply(Input::Request(R::Redraw))
            .unwrap()
            .mutations
            .contains(&Mutation::ClearScreen)
    );
    assert_eq!(
        engine.apply(Input::Request(R::Interrupt)).unwrap().event,
        Some(Event::Interrupted)
    );
    assert_eq!(engine.editor.text(), "hello界");
    engine
        .start(Prompt::new("again").unwrap(), (80, 24))
        .unwrap();
    // Physical Ctrl-D binding deletes on nonempty; transport EOF always ends.
    assert!(
        engine
            .apply(Input::Request(R::DeleteOrEof))
            .unwrap()
            .event
            .is_none()
    );
    assert_eq!(engine.editor.text(), "hello");
    assert_eq!(
        engine.apply(Input::TransportEof).unwrap().event,
        Some(Event::EndOfInput)
    );
    engine.editor.clear();
    engine
        .start(Prompt::new("again").unwrap(), (80, 24))
        .unwrap();
    assert_eq!(
        engine.apply(Input::Request(R::DeleteOrEof)).unwrap().event,
        Some(Event::EndOfInput)
    );
    assert!(matches!(
        engine.apply(Input::Text("x".into())),
        Err(Error::State)
    ));
}

#[test]
fn virtual_transport_paste_output_restore_and_independent_instances() {
    let a = Virtual::new();
    let b = Virtual::new();
    let (mut e, mut t) = start(&a);
    let (mut other, mut second) = start(&b);
    feed(&a, &mut t, &mut e, b"\x1b[200~a\r\nb\rc\x1b[201~\x1b[D");
    assert_eq!((e.editor.text(), e.editor.cursor()), ("a\nb\nc", 4));
    let effects = e.external_output(Role::Dim, "notice").unwrap();
    t.apply(&mut e, effects).unwrap();
    assert_eq!((e.editor.text(), e.editor.cursor()), ("a\nb\nc", 4));
    assert!(
        a.screen()
            .screen()
            .contents()
            .contains("notice\ndemo> a\n... b\n... c")
    );
    assert_eq!(a.screen().screen().cursor_position(), (3, 4));
    a.0.borrow_mut().size = (10, 5);
    t.poll(&mut e, Duration::ZERO).unwrap();
    assert_eq!(e.editor.text(), "a\nb\nc");
    t.close(&mut e).unwrap();
    assert!(other.is_open());
    second.close(&mut other).unwrap();
    for d in [a, b] {
        let data = d.0.borrow();
        assert_eq!(data.restored, 1);
        assert_eq!(
            data.output
                .windows(8)
                .filter(|w| *w == b"\x1b[?2004h")
                .count(),
            data.output
                .windows(8)
                .filter(|w| *w == b"\x1b[?2004l")
                .count()
        );
    }
}

#[test]
fn transport_faults_cleanup_drop_and_reopen_share_one_driver() {
    for read_error in [true, false] {
        let d = Virtual::new();
        let (mut e, mut t) = start(&d);
        e.complete(0..0, "draft").unwrap();
        if read_error {
            d.0.borrow_mut().fail_read = true;
        } else {
            d.0.borrow_mut().fail_write = true;
            d.feed(b"a");
        }
        assert!(t.poll(&mut e, Duration::ZERO).is_err());
        assert!(!e.is_open() && !t.active);
        assert_eq!(d.0.borrow().restored, 1);
        drop(t);
        d.0.borrow_mut().fail_read = false;
        d.0.borrow_mut().fail_write = false;
        let mut t = Terminal::start(
            d.clone(),
            &mut e,
            Prompt::new("again").unwrap(),
            Theme::new(false, false, None),
        )
        .unwrap();
        t.close(&mut e).unwrap();
        assert_eq!(d.0.borrow().restored, 2);
    }
    let d = Virtual::new();
    d.0.borrow_mut().fail_write = true;
    let mut e = Engine::new(Editor::new(10, 0));
    assert!(
        Terminal::start(
            d.clone(),
            &mut e,
            Prompt::new("demo").unwrap(),
            Theme::new(false, false, None)
        )
        .is_err()
    );
    assert!(!e.is_open());
    assert_eq!(d.0.borrow().restored, 1);
    let d = Virtual::new();
    let (_e, t) = start(&d);
    drop(t);
    assert_eq!(d.0.borrow().restored, 1);
    let d = Virtual::new();
    let (mut e, mut t) = start(&d);
    d.0.borrow_mut().fail_restore = 1;
    assert!(t.close(&mut e).is_err());
    assert!(t.active);
    drop(t);
    assert_eq!(d.0.borrow().restored, 1);
}

#[test]
fn driven_idle_resize_and_output_have_explicit_work_ownership() {
    use crate::{WaitInterest, Wake};
    let d = Virtual::new();
    let (mut e, mut t) = start(&d);
    for _ in 0..10_000 {
        assert_eq!(t.interest(), WaitInterest::Input { deadline: None });
    }
    assert_eq!(
        (
            d.0.borrow().reads,
            d.0.borrow().writes,
            d.0.borrow().dimension_queries
        ),
        (0, 1, 1)
    );
    d.feed("e\u{301}界\x1b[D!".as_bytes());
    t.advance(&mut e, Wake::InputReady).unwrap();
    assert_eq!((e.editor.text(), e.editor.cursor()), ("e\u{301}!界", 4));
    let fx = e.external_output(Role::Dim, "application event").unwrap();
    t.apply(&mut e, fx).unwrap();
    assert_eq!(d.0.borrow().dimension_queries, 1);
    d.0.borrow_mut().size = (12, 24);
    t.advance(&mut e, Wake::Resize).unwrap();
    assert_eq!(d.0.borrow().dimension_queries, 2);
    assert_eq!((e.editor.text(), e.editor.cursor()), ("e\u{301}!界", 4));
    assert!(d.0.borrow().waits.iter().all(Duration::is_zero));
    assert_eq!(t.interest(), WaitInterest::Input { deadline: None });
    t.close(&mut e).unwrap();
    assert_eq!(d.0.borrow().restored, 1);
}

#[test]
fn driven_deadlines_are_early_stale_and_session_safe() {
    use crate::{WaitInterest, Wake};
    fn deadline(t: &Terminal<Virtual>) -> crate::Deadline {
        let WaitInterest::Input {
            deadline: Some(token),
        } = t.interest()
        else {
            panic!("no deadline")
        };
        token
    }
    let d = Virtual::new();
    let (mut e, mut t) = start(&d);
    d.feed(b"ab\x1b[");
    t.advance(&mut e, Wake::InputReady).unwrap();
    let early = deadline(&t);
    let reads = d.0.borrow().reads;
    assert_eq!(t.advance(&mut e, Wake::Deadline(early)).unwrap(), None);
    assert_eq!(d.0.borrow().reads, reads);
    d.feed(b"D!");
    t.advance(&mut e, Wake::InputReady).unwrap();
    assert_eq!(e.editor.text(), "a!b");
    assert_eq!(t.advance(&mut e, Wake::Deadline(early)).unwrap(), None);
    d.feed(b"\x1b[");
    t.advance(&mut e, Wake::InputReady).unwrap();
    t.expire_for_test();
    let due = deadline(&t);
    assert_eq!(
        t.advance(&mut e, Wake::Deadline(due)).unwrap(),
        Some(Event::Rejected(crate::EditError::InvalidSequence))
    );
    assert_eq!(t.advance(&mut e, Wake::Deadline(due)).unwrap(), None);
    t.close(&mut e).unwrap();
    let (mut e, mut t) = start(&d);
    d.feed(b"\x1b[");
    t.advance(&mut e, Wake::InputReady).unwrap();
    assert_eq!(t.advance(&mut e, Wake::Deadline(due)).unwrap(), None);
    assert!(matches!(
        t.interest(),
        WaitInterest::Input { deadline: Some(_) }
    ));
    // Even a due notification reconciles ready continuation before expiry.
    t.expire_for_test();
    let due = deadline(&t);
    d.feed(b"D");
    assert_eq!(t.advance(&mut e, Wake::Deadline(due)).unwrap(), None);
    assert_eq!(t.interest(), WaitInterest::Input { deadline: None });
}

#[test]
fn driven_semantic_boundary_retains_read_ahead_and_cleans_failures() {
    use crate::{WaitInterest, Wake};
    let d = Virtual::new();
    let (mut e, mut t) = start(&d);
    d.feed(b"he\tllo\rnext");
    assert_eq!(
        t.advance(&mut e, Wake::InputReady).unwrap(),
        Some(Event::CompletionRequested)
    );
    assert_eq!(t.interest(), WaitInterest::Ready);
    assert_eq!(e.editor.text(), "he");
    assert_eq!(
        t.advance(&mut e, Wake::InputReady).unwrap(),
        Some(Event::Submitted("hello".into()))
    );
    assert_eq!(t.pending.iter().copied().collect::<Vec<_>>(), b"next");
    assert_eq!(d.0.borrow().restored, 1);
    for fail_read in [true, false] {
        let d = Virtual::new();
        let (mut e, mut t) = start(&d);
        d.0.borrow_mut().fail_read = fail_read;
        d.0.borrow_mut().fail_write = !fail_read;
        d.feed(b"x");
        assert!(t.advance(&mut e, Wake::InputReady).is_err());
        assert!(!t.active);
        assert!(!e.is_open());
        assert_eq!(d.0.borrow().restored, 1);
    }
}

#[test]
fn optional_paste_admission_controls_every_transaction_and_restoration() {
    let d = Virtual::new();
    let mut e = Engine::new(Editor::new(100, 1));
    let mut t = Terminal::start_config(
        d.clone(),
        &mut e,
        Prompt::new("demo").unwrap(),
        Theme::new(true, true, None),
        crate::InteractionFeatures {
            styling: false,
            bracketed_paste: false,
        },
    )
    .unwrap();
    let fx = e.external_output(Role::Dim, "notice").unwrap();
    t.apply(&mut e, fx).unwrap();
    t.close(&mut e).unwrap();
    assert!(!d.0.borrow().output.windows(6).any(|s| s == b"[?2004"));
    assert_eq!(d.0.borrow().restored, 1);
}

#[test]
fn public_scheduling_values_are_portable_and_single_owner_mutation_is_sufficient() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<crate::Interaction>();
    send_sync::<crate::Deadline>();
    send_sync::<crate::WaitInterest>();
    send_sync::<crate::Wake>();
    send_sync::<crate::TerminalConfig>();
}

#[test]
fn analysis_lifecycle_output_resize_and_stale_effects() {
    use crate::AnalysisOutcome;
    let mut e = Engine::new(Editor::new(256, 4));
    e.editor.insert("draft界").unwrap();
    e.editor.admit_history("previous").unwrap();
    let original = e.editor.analysis_snapshot();
    e.start(Prompt::new("analysis").unwrap(), (80, 24)).unwrap();
    e.apply(Input::Resize(40, 20)).unwrap();
    e.external_output(Role::Warning, "notice").unwrap();
    e.output_document(
        &crate::Document::new(vec![crate::Block::Paragraph(
            crate::Text::new("record").unwrap(),
        )])
        .unwrap(),
    )
    .unwrap();
    e.close();
    assert_eq!(original.revision(), e.editor.revision());
    e.start(Prompt::new("again").unwrap(), (80, 24)).unwrap();
    assert_eq!(original.revision(), e.editor.revision());
    e.apply(Input::Edit(E::Left)).unwrap();
    let current = e.editor.analysis_snapshot();
    let (outcome, effects) = e
        .complete_at(original.revision(), 0..usize::MAX, "\x1b")
        .unwrap();
    assert_eq!(outcome, AnalysisOutcome::Stale);
    assert!(effects.mutations.is_empty() && effects.event.is_none());
    assert_eq!(
        (e.editor.text(), e.editor.cursor(), e.editor.revision()),
        (current.text(), current.cursor(), current.revision())
    );
    for request in [R::Submit, R::Interrupt] {
        let old = e.editor.revision();
        e.apply(Input::Request(request)).unwrap();
        assert_ne!(old, e.editor.revision());
        e.start(Prompt::new("again").unwrap(), (80, 24)).unwrap();
        assert_eq!(
            e.complete_at(old, 0..0, "bad").unwrap().0,
            AnalysisOutcome::Stale
        );
    }
    let old = e.editor.revision();
    e.apply(Input::TransportEof).unwrap();
    assert_ne!(old, e.editor.revision());
}

#[test]
fn analysis_decoder_atomicity_and_virtual_transport_output() {
    let d = Virtual::new();
    let (mut e, mut t) = start(&d);
    let r = e.editor.revision();
    d.feed(b"\x1b[");
    t.poll(&mut e, Duration::ZERO).unwrap();
    assert_eq!(r, e.editor.revision());
    d.feed(b"D");
    t.poll(&mut e, Duration::ZERO).unwrap(); // Left at zero is a no-op.
    assert_eq!(r, e.editor.revision());
    d.feed(b"\xff");
    t.poll(&mut e, Duration::ZERO).unwrap();
    assert_eq!(r, e.editor.revision());
    d.feed(b"\x1b[200~a\r\n");
    t.poll(&mut e, Duration::ZERO).unwrap();
    assert_eq!(r, e.editor.revision());
    d.feed("界\x1b[201~".as_bytes());
    t.poll(&mut e, Duration::ZERO).unwrap();
    // Paste control filtering/normalization is the existing decoder contract.
    let mut expected = r;
    expected.advance();
    assert_eq!(expected, e.editor.revision(), "one atomic paste edit");
    let now = e.editor.analysis_snapshot();
    let writes = d.0.borrow().writes;
    let (result, effects) = e.complete_at(r, 0..0, "bad").unwrap();
    assert_eq!(result, crate::AnalysisOutcome::Stale);
    assert!(effects.mutations.is_empty());
    assert_eq!(writes, d.0.borrow().writes);
    assert_eq!(now.revision(), e.editor.revision());
}

#[test]
fn completion_virtual_transport_preserves_screen_and_cleans_write_failure() {
    use crate::{CompletionAction, CompletionCandidate, CompletionSet};
    for plain in [false, true] {
        let d = Virtual::new();
        let mut e = Engine::new(Editor::new(128, 3));
        e.editor.insert("bu").unwrap();
        let theme = Theme::new(!plain, true, Some("xterm"));
        let mut t =
            Terminal::start(d.clone(), &mut e, Prompt::new("demo").unwrap(), theme).unwrap();
        let old = e.editor.revision();
        let set = CompletionSet::new(
            old,
            vec![
                CompletionCandidate::new(0..2, "build", "build").unwrap(),
                CompletionCandidate::new(0..2, "bundle", "bundle").unwrap(),
            ],
        )
        .unwrap();
        let (_, fx) = e.present_completions(set).unwrap();
        t.apply(&mut e, fx).unwrap();
        assert!(d.screen().screen().contents().contains("> build"));
        d.feed(b"\t");
        t.advance(&mut e, crate::Wake::InputReady).unwrap();
        assert_eq!(e.editor.revision(), old);
        assert!(d.screen().screen().contents().contains("> bundle"));
        let fx = e.external_output(Role::Warning, "host output").unwrap();
        t.apply(&mut e, fx).unwrap();
        assert!(d.screen().screen().contents().contains("> bundle"));
        assert_eq!(d.screen().screen().cursor_position(), (1, 8));
        let (_, fx) = e.completion_action(CompletionAction::Accept).unwrap();
        t.apply(&mut e, fx).unwrap();
        assert_eq!(e.editor.text(), "bundle");
        assert!(!d.screen().screen().contents().contains("> build"));
        let set = CompletionSet::new(
            e.editor.revision(),
            vec![CompletionCandidate::new(0..6, "x", "x").unwrap()],
        )
        .unwrap();
        let (_, fx) = e.present_completions(set).unwrap();
        d.0.borrow_mut().fail_write = true;
        assert!(t.apply(&mut e, fx).is_err());
        assert!(!e.is_open() && e.completion_selection().is_none());
        assert_eq!(d.0.borrow().restored, 1);
    }
}
