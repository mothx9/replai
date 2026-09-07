//! Linux-qualified POSIX resource realization. No editor, prompt or key policy.
use crate::{
    Error,
    substrate::{Read, Transport},
};
use rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    io::{dup, read, write},
    termios::{self, OptionalActions, Termios},
};
use std::{
    io,
    os::fd::{AsFd, OwnedFd},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
static ACTIVE: AtomicBool = AtomicBool::new(false);
struct Lease;
impl Lease {
    fn acquire() -> io::Result<Self> {
        ACTIVE
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "another terminal interaction is active in this process",
                )
            })?;
        Ok(Self)
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        ACTIVE.store(false, Ordering::Release);
    }
}

pub(crate) struct Resource {
    input: OwnedFd,
    pub(crate) output: OwnedFd,
    saved: Termios,
    active: bool,
    lease: Option<Lease>,
}
impl Resource {
    pub fn acquire(input: &impl AsFd, output: &impl AsFd) -> Result<(Self, (usize, usize)), Error> {
        if !termios::isatty(input) || !termios::isatty(output) {
            return Err(Error::UnsuitableTerminal);
        }
        let istat = rustix::fs::fstat(input).map_err(io::Error::from)?;
        let ostat = rustix::fs::fstat(output).map_err(io::Error::from)?;
        if istat.st_rdev != ostat.st_rdev {
            return Err(Error::UnsuitableTerminal);
        }
        let lease = Lease::acquire().map_err(|_| Error::Busy)?;
        let input = dup(input).map_err(io::Error::from)?;
        let output = dup(output).map_err(io::Error::from)?;
        let saved = termios::tcgetattr(&input).map_err(io::Error::from)?;
        let size = dimensions(&output).map_err(|e| {
            if e.kind() == io::ErrorKind::Unsupported {
                Error::UnsuitableTerminal
            } else {
                Error::Io(e)
            }
        })?;

        let mut resource = Self {
            input,
            output,
            saved,
            active: true,
            lease: Some(lease),
        };
        let mut raw = resource.saved.clone();
        raw.make_raw();
        if let Err(e) = termios::tcsetattr(&resource.input, OptionalActions::Now, &raw) {
            let error = io::Error::from(e);
            return Err(match resource.restore() {
                Ok(()) => error.into(),
                Err(cleanup) => io::Error::new(
                    error.kind(),
                    format!("{error}; terminal cleanup also failed: {cleanup}"),
                )
                .into(),
            });
        }
        Ok((resource, size))
    }
}
impl Transport for Resource {
    fn dimensions(&self) -> io::Result<(usize, usize)> {
        dimensions(&self.output)
    }
    fn write(&self, bytes: &[u8]) -> io::Result<()> {
        write_all(&self.output, bytes)
    }
    fn cleanup_write(&self, bytes: &[u8]) -> io::Result<()> {
        let result = self.write(bytes);
        if result.is_err() {
            let _ = write_all(&self.input, bytes);
        }
        result
    }
    fn read(&self, timeout: Duration) -> io::Result<Read> {
        let ts = Timespec {
            tv_sec: timeout.as_secs().try_into().unwrap_or(i64::MAX),
            tv_nsec: timeout.subsec_nanos().into(),
        };
        let mut fds = [PollFd::new(&self.input, PollFlags::IN)];
        match poll(&mut fds, Some(&ts)) {
            Ok(0) | Err(rustix::io::Errno::INTR) => return Ok(Read::Idle),
            Ok(_) => {}
            Err(e) => return Err(e.into()),
        }
        let mut byte = [0];
        match read(&self.input, &mut byte) {
            Ok(0) => Ok(Read::Eof),
            Ok(_) => Ok(Read::Byte(byte[0])),
            Err(rustix::io::Errno::INTR | rustix::io::Errno::AGAIN) => Ok(Read::Idle),
            Err(e) => Err(e.into()),
        }
    }
    fn restore(&mut self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }
        termios::tcsetattr(&self.input, OptionalActions::Now, &self.saved)
            .map_err(io::Error::from)?;
        self.active = false;
        self.lease.take();
        Ok(())
    }
}
impl Drop for Resource {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}
fn dimensions(output: &impl AsFd) -> io::Result<(usize, usize)> {
    let size = termios::tcgetwinsize(output)?;
    if size.ws_col < 2 || size.ws_row < 2 {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "terminal must provide at least 2 columns and 2 rows",
        ));
    }
    Ok((size.ws_col.into(), size.ws_row.into()))
}
fn write_all(output: &impl AsFd, mut bytes: &[u8]) -> io::Result<()> {
    while !bytes.is_empty() {
        match write(output, bytes) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "terminal write made no progress",
                ));
            }
            Ok(n) => bytes = &bytes[n..],
            Err(rustix::io::Errno::INTR) => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}
