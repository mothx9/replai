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
}
#[derive(Clone)]
struct Virtual(Rc<RefCell<Device>>);
impl Virtual {
    fn new() -> Self {
        Self(Rc::new(RefCell::new(Device {
            size: (80, 24),
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
        Ok(self.0.borrow().size)
    }
    fn read(&self, _: Duration) -> io::Result<Read> {
        let mut d = self.0.borrow_mut();
        if d.fail_read {
            return Err(io::Error::other("read failure"));
        }
        Ok(d.input
            .pop_front()
            .map_or(if d.eof { Read::Eof } else { Read::Idle }, Read::Byte))
    }
    fn write(&self, bytes: &[u8]) -> io::Result<()> {
        let mut d = self.0.borrow_mut();
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
