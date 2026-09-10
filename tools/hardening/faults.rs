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
        let n = buffer.len().min(d.bytes.len());
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
    let revision = e.editor.revision();
    let fx = e.apply(Input::Resize(20, 4)).unwrap();
    t.apply(&mut e, fx).unwrap();
    assert_eq!(e.editor.revision(), revision);
    t.close(&mut e).unwrap();
    println!(
        "{{\"virtual_failure_hits\":{exercised:?},\"repetitions\":{repeats},\"idle_queries\":100000}}"
    );
}
