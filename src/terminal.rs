//! Compatibility VT driver over a byte transport; timing belongs here, not in Engine.
use crate::{
    Error, Event, Prompt, Theme,
    actions::Input,
    engine::{Effects, Engine},
    input::Decoder,
    keymap::binding,
    protocol::encode,
    render::Mutation,
    substrate::{Read, Transport, combine},
};
use std::{
    collections::VecDeque,
    io,
    time::{Duration, Instant},
};

const SEQUENCE_IDLE: Duration = Duration::from_millis(250);
const READ_BYTES: usize = 4096;
const READY_BYTES: usize = 32 * 1024;
const READY_TIME: Duration = Duration::from_millis(2);
pub(crate) struct Terminal<T: Transport> {
    pub(crate) resource: T,
    pub(crate) active: bool,
    theme: Theme,
    decoder: Decoder,
    last_byte: Instant,
    // Only the unread tail at a host-visible boundary; ordinary reads use stack
    // storage. The Interaction retains this bounded tail across close/reopen.
    pub(crate) pending: VecDeque<u8>,
}
impl<T: Transport> Terminal<T> {
    #[cfg(test)]
    pub(crate) fn expire_for_test(&mut self) {
        self.last_byte = Instant::now() - SEQUENCE_IDLE;
    }
    pub fn start(
        resource: T,
        engine: &mut Engine,
        prompt: Prompt,
        theme: Theme,
    ) -> Result<Self, Error> {
        let mut terminal = Self {
            resource,
            active: true,
            theme,
            decoder: Decoder::new(engine.editor.capacity()),
            last_byte: Instant::now(),
            pending: VecDeque::new(),
        };
        let result = (|| {
            let size = terminal.resource.dimensions()?;
            let mut effects = engine.start(prompt, size)?;
            effects.mutations.insert(0, Mutation::Paste(true));
            terminal.apply(engine, effects)?;
            Ok(())
        })();
        if let Err(error) = result {
            engine.abandon();
            return Err(terminal.failure(error));
        }
        Ok(terminal)
    }
    pub fn apply(&mut self, engine: &mut Engine, effects: Effects) -> Result<Option<Event>, Error> {
        if effects.mutations.is_empty() && engine.is_open() {
            return Ok(effects.event);
        }
        let output = self
            .resource
            .write(encode(&effects.mutations, self.theme).as_bytes());
        if !engine.is_open() {
            let restored = self.restore();
            return combine(output, restored)
                .map(|()| effects.event)
                .map_err(Error::Io);
        }
        if let Err(error) = output {
            engine.abandon();
            return Err(self.failure(error.into()));
        }
        Ok(effects.event)
    }
    pub fn poll(&mut self, engine: &mut Engine, timeout: Duration) -> Result<Option<Event>, Error> {
        let result = self.poll_inner(engine, timeout);
        if result.is_err() {
            engine.abandon();
        }
        result.map_err(|error| self.failure(error))
    }
    fn poll_inner(
        &mut self,
        engine: &mut Engine,
        timeout: Duration,
    ) -> Result<Option<Event>, Error> {
        let size = self.resource.dimensions()?;
        let effects = engine.apply(Input::Resize(size.0, size.1))?;
        self.apply(engine, effects)?;
        let mut buffer = [0; READ_BYTES];
        let (mut position, mut length) = (0, 0);
        let mut consumed = 0;
        let mut began = None;
        let mut wait = timeout.min(Duration::from_millis(100));
        if !self.pending.is_empty() {
            self.last_byte = Instant::now();
            wait = Duration::ZERO;
        }
        let result = (|| loop {
            let byte = if let Some(byte) = self.pending.pop_front() {
                byte
            } else if position < length {
                let byte = buffer[position];
                position += 1;
                byte
            } else {
                match self.resource.read(&mut buffer, wait)? {
                    Read::Bytes(n) => {
                        assert!(n > 0 && n <= buffer.len());
                        self.last_byte = Instant::now();
                        (position, length) = (1, n);
                        buffer[0]
                    }
                    Read::Idle => {
                        if self.decoder.pending()
                            && self.last_byte.elapsed() >= SEQUENCE_IDLE
                            && let Some(key) = self.decoder.expire()
                        {
                            let input = binding(key)
                                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                            let effects = engine.apply(input)?;
                            return self.apply(engine, effects);
                        }
                        let effects = engine.flush();
                        return self.apply(engine, effects);
                    }
                    Read::Eof => {
                        if self.decoder.pending() {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "end of input during an incomplete sequence",
                            )
                            .into());
                        }
                        let effects = engine.apply(Input::TransportEof)?;
                        return self.apply(engine, effects);
                    }
                }
            };
            // Drain only input which is ready now. No debounce or collection wait.
            wait = Duration::ZERO;
            let started = *began.get_or_insert_with(Instant::now);
            consumed += 1;
            let mut time_exhausted = false;
            if let Some(key) = self.decoder.feed(byte) {
                let input =
                    binding(key).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                let effects = engine.apply_deferred(input)?;
                if let Some(event) = self.apply(engine, effects)? {
                    return Ok(Some(event));
                }
                time_exhausted = started.elapsed() >= READY_TIME;
            }
            if consumed >= READY_BYTES || time_exhausted {
                let effects = engine.flush();
                return self.apply(engine, effects);
            }
        })();
        // Preserve read-ahead even on rejection, submit, interrupt or I/O failure.
        // New reads occur only after the previous tail is fully consumed.
        self.pending.extend(&buffer[position..length]);
        debug_assert!(self.pending.len() <= READ_BYTES);
        result
    }
    pub fn close(&mut self, engine: &mut Engine) -> Result<(), Error> {
        if !self.active {
            engine.abandon();
            return Ok(());
        }
        let effects = engine.close();
        self.apply(engine, effects).map(|_| ())
    }
    fn restore(&mut self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }
        let bytes = encode(
            &[
                Mutation::Paste(false),
                Mutation::Style(crate::Role::Default),
            ],
            self.theme,
        );
        let output = self.resource.cleanup_write(bytes.as_bytes());
        let restored = self.resource.restore();
        if restored.is_ok() {
            self.active = false;
        }
        combine(output, restored)
    }
    fn failure(&mut self, error: Error) -> Error {
        match self.restore() {
            Ok(()) => error,
            Err(cleanup) => Error::Io(io::Error::other(format!(
                "{error}; terminal cleanup also failed: {cleanup}"
            ))),
        }
    }
}
impl<T: Transport> Drop for Terminal<T> {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
#[path = "../tests/support/posix_pty.rs"]
pub(crate) mod pty_support;

#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
mod tests {
    use super::*;
    use crate::{Editor, Role, system::Resource};
    use rustix::io::read;
    #[test]
    fn write_failure_during_active_output_restores_termios_and_paste_mode() {
        use rustix::{
            fs::{Mode, OFlags, open},
            termios::{Winsize, tcgetattr, tcsetwinsize, ttyname},
        };
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
        let before = format!("{:?}", tcgetattr(&slave).unwrap());
        let mut editor = Editor::new(100, 1);
        editor.insert("draft").unwrap();
        editor.left();
        let (resource, _) = Resource::acquire(&slave, &slave).unwrap();
        let mut engine = Engine::new(editor);
        let mut t = Terminal::start(
            resource,
            &mut engine,
            Prompt::new("demo").unwrap(),
            Theme::new(true, false, None),
        )
        .unwrap();
        let mut bytes = [0; 4096];
        read(&master, &mut bytes).unwrap();
        // Replace only the owned output FD with a real read-only handle to the
        // same PTY, after successful acquisition. No mocked writes or cleanup.
        t.resource.output = open(
            ttyname(&slave, Vec::new()).unwrap(),
            OFlags::RDONLY | OFlags::NOCTTY,
            Mode::empty(),
        )
        .unwrap();
        assert!(matches!(
            {
                let effects = engine.external_output(Role::Dim, "notice").unwrap();
                t.apply(&mut engine, effects)
            },
            Err(Error::Io(_))
        ));
        assert_eq!(format!("{:?}", tcgetattr(&slave).unwrap()), before);
        assert_eq!((engine.editor.text(), engine.editor.cursor()), ("draft", 4));
        let n = read(&master, &mut bytes).unwrap();
        assert!(bytes[..n].windows(8).any(|w| w == b"\x1b[?2004l"));
        assert!(!t.active);
    }
}
