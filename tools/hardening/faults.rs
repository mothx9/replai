use crate::{
    actions::Input,
    engine::Engine,
    substrate::{Read, Transport},
    terminal::Terminal,
    *,
};
use std::{cell::RefCell, collections::VecDeque, io, rc::Rc, time::Duration};

#[derive(Default)]
struct Device {
    bytes: VecDeque<u8>,
    output: Vec<u8>,
    calls: [usize; 4],
    fail: Option<(usize, usize)>,
    restored: bool,
    chunk: Option<usize>,
}
#[derive(Clone, Default)]
struct Virtual(Rc<RefCell<Device>>);
impl Virtual {
    fn call(&self, kind: usize) -> io::Result<()> {
        let mut d = self.0.borrow_mut();
        d.calls[kind] += 1;
        if d.fail == Some((kind, d.calls[kind])) {
            return Err(io::Error::other("injected boundary failure"));
        }
        Ok(())
    }
}
impl Transport for Virtual {
    fn dimensions(&self) -> io::Result<(usize, usize)> {
        self.call(0)?;
        Ok((40, 10))
    }
    fn read(&self, buffer: &mut [u8], timeout: Duration) -> io::Result<Read> {
        assert_eq!(timeout, Duration::ZERO);
        self.call(1)?;
        let mut d = self.0.borrow_mut();
        let n = buffer.len().min(d.bytes.len()).min(d.chunk.unwrap_or(4096));
        for b in &mut buffer[..n] {
            *b = d.bytes.pop_front().unwrap();
        }
        Ok(if n == 0 { Read::Idle } else { Read::Bytes(n) })
    }
    fn write(&self, bytes: &[u8]) -> io::Result<()> {
        self.call(2)?;
        self.0.borrow_mut().output.extend_from_slice(bytes);
        Ok(())
    }
    fn cleanup_write(&self, bytes: &[u8]) -> io::Result<()> {
        self.write(bytes)
    }
    fn restore(&mut self) -> io::Result<()> {
        self.call(3)?;
        self.0.borrow_mut().restored = true;
        Ok(())
    }
}

/// Sweep actual generic transport calls, including acquisition and cleanup.
/// Faults are one-shot so a failed restoration has an observable retry route.
pub fn fault_campaign(repeats: usize) {
    assert!(repeats <= 10_000);
    let mut exercised = [0; 4];
    for (kind, hits) in exercised.iter_mut().enumerate() {
        for position in 1..=8 {
            for _ in 0..repeats {
                let d = Virtual::default();
                d.0.borrow_mut().fail = Some((kind, position));
                let mut e = Engine::new(Editor::new(256, 8));
                if let Ok(mut t) = Terminal::start(
                    d.clone(),
                    &mut e,
                    Prompt::new("fault").unwrap(),
                    Theme::new(false, false, None),
                ) {
                    for _ in 0..4 {
                        if !t.active {
                            break;
                        }
                        d.0.borrow_mut().bytes.extend(b"x");
                        if t.advance(&mut e, Wake::InputReady).is_err() {
                            break;
                        }
                        if t.advance(&mut e, Wake::Resize).is_err() {
                            break;
                        }
                        let fx = e.external_output(Role::Default, "notice").unwrap();
                        if t.apply(&mut e, fx).is_err() {
                            break;
                        }
                    }
                    let _ = t.close(&mut e);
                    drop(t);
                }
                let device = d.0.borrow();
                if device.calls[kind] >= position {
                    *hits += 1;
                }
                assert!(
                    device.restored,
                    "restoration retry failed for kind {kind}, call {position}"
                );
                assert!(!e.is_open());
            }
        }
        assert!(*hits >= repeats);
    }
    // Idle queries are pure; they cannot become a compensating timer or I/O call.
    let d = Virtual::default();
    let mut e = Engine::new(Editor::new(256, 0));
    let mut t = Terminal::start(
        d.clone(),
        &mut e,
        Prompt::new("idle").unwrap(),
        Theme::new(false, false, None),
    )
    .unwrap();
    let before = d.0.borrow().calls;
    for _ in 0..100_000 {
        assert_eq!(t.interest(), WaitInterest::Input { deadline: None });
    }
    assert_eq!(d.0.borrow().calls, before);
    // Serialized output is exactly one write transaction. A stale result is none.
    let revision = e.editor.revision();
    let writes = d.0.borrow().calls[2];
    let fx = e.external_output(Role::Success, "host notice").unwrap();
    t.apply(&mut e, fx).unwrap();
    assert_eq!(d.0.borrow().calls[2], writes + 1);
    assert_eq!(e.editor.revision(), revision);
    e.editor.insert("x").unwrap();
    let (outcome, fx) = e
        .present_analysis(
            AnalysisPresentation::new(revision, vec![], Some(Hint::new("old", Role::Dim).unwrap()))
                .unwrap(),
        )
        .unwrap();
    assert_eq!(outcome, AnalysisOutcome::Stale);
    let calls = d.0.borrow().calls;
    t.apply(&mut e, fx).unwrap();
    assert_eq!(d.0.borrow().calls, calls);
    let revision = e.editor.revision();
    let fx = e.apply(Input::Resize(20, 4)).unwrap();
    t.apply(&mut e, fx).unwrap();
    assert_eq!(e.editor.revision(), revision);
    t.close(&mut e).unwrap();
    println!(
        "{{\"virtual_failure_hits\":{exercised:?},\"repetitions\":{repeats},\"idle_queries\":100000}}"
    );
}

/// Real generic driver, arbitrary read fragmentation, no waits, and both paste policies.
pub fn protocol_transport_case(data: &[u8]) {
    for chunk in [1, 7, 4096] {
        for bracketed_paste in [false, true] {
            let d = Virtual::default();
            d.0.borrow_mut().chunk = Some(chunk);
            d.0.borrow_mut().bytes.extend(data);
            let mut e = Engine::new(Editor::new(4096, 8));
            let mut t = Terminal::start_config(
                d.clone(),
                &mut e,
                Prompt::new("protocol").unwrap(),
                Theme::new(false, false, None),
                InteractionFeatures {
                    styling: false,
                    bracketed_paste,
                },
            )
            .unwrap();
            let mut finished = false;
            for _ in 0..=data.len() + 1 {
                if !t.active {
                    finished = true;
                    break;
                }
                let revision = e.editor.revision();
                t.advance(&mut e, Wake::Resize).unwrap();
                assert_eq!(revision, e.editor.revision());
                if t.advance(&mut e, Wake::InputReady).is_err() {
                    finished = true;
                    break;
                }
                if d.0.borrow().bytes.is_empty() && !matches!(t.interest(), WaitInterest::Ready) {
                    // The host can inspect a deadline without I/O. Don't sleep in a fuzzer.
                    let calls = d.0.borrow().calls;
                    let interest = t.interest();
                    assert_eq!(calls, d.0.borrow().calls);
                    if let WaitInterest::Input {
                        deadline: Some(token),
                    } = interest
                    {
                        // A token from another session can never be applied here.
                        let stale = Deadline {
                            session: 0,
                            ..token
                        };
                        assert_eq!(t.advance(&mut e, Wake::Deadline(stale)).unwrap(), None);
                        assert_eq!(calls, d.0.borrow().calls);
                    }
                    finished = true;
                    break;
                }
            }
            assert!(
                finished,
                "driver must consume bounded input or report a terminal failure"
            );
            assert!(e.editor.text().len() <= e.editor.capacity());
            let fx = e.close();
            t.apply(&mut e, fx).unwrap();
            drop(t);
            assert!(d.0.borrow().restored);
            assert!(d.0.borrow().output.len() < 8 * 1024 * 1024);
        }
    }
}
