//! Shared Linux/macOS POSIX resource realization. No editor, prompt or key policy.
use crate::{
    Error, InteractionRequirements, TerminalCapabilities, TerminalConfig, TerminalRealization,
    Theme,
    substrate::{Read, Transport},
};
#[cfg(target_os = "macos")]
use nix::sys::event::{EvFlags, EventFilter, FilterFlag, KEvent as Readiness, Kqueue};
#[cfg(target_os = "linux")]
use rustix::event::{PollFd, PollFlags, Timespec, poll};
use rustix::{
    io::{dup, read, write},
    termios::{self, OptionalActions, Termios},
};
#[cfg(target_os = "macos")]
use std::os::fd::AsRawFd;
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
    #[cfg(target_os = "macos")]
    readiness: Option<Kqueue>,
}
impl Resource {
    pub(crate) fn input_source(&self) -> std::os::fd::BorrowedFd<'_> {
        self.input.as_fd()
    }

    pub fn acquire(
        input: &impl AsFd,
        output: &impl AsFd,
        config: TerminalConfig,
        requirements: InteractionRequirements,
    ) -> Result<(Self, Theme, TerminalCapabilities), Error> {
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
                Error::CapabilityMismatch(
                    "terminal dimensions of at least 2 columns and 2 rows required",
                )
            } else {
                Error::Io(e)
            }
        })?;

        // Observed resource facts cannot be replaced by host protocol assumptions.
        // Resolve before any raw-mode mutation; failed admission drops only duplicates.
        let (theme, capabilities) =
            config.resolve_for(TerminalRealization::interactive(size), requirements)?;
        let mut resource = Self {
            input,
            output,
            saved,
            active: true,
            lease: Some(lease),
            #[cfg(target_os = "macos")]
            readiness: None,
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
        #[cfg(target_os = "macos")]
        if let Err(error) = resource.prepare_readiness() {
            return Err(match resource.restore() {
                Ok(()) => error.into(),
                Err(cleanup) => io::Error::new(
                    error.kind(),
                    format!("{error}; terminal cleanup also failed: {cleanup}"),
                )
                .into(),
            });
        }
        Ok((resource, theme, capabilities))
    }
}
impl Resource {
    #[cfg(target_os = "macos")]
    fn prepare_readiness(&mut self) -> io::Result<()> {
        // Register after raw mode is installed: an existing canonical knote can
        // otherwise miss bytes already queued when ICANON changes on Darwin.
        let queue = Kqueue::new()?;
        rustix::io::fcntl_setfd(&queue, rustix::io::FdFlags::CLOEXEC)?;
        let event = Readiness::new(
            self.input.as_raw_fd() as usize,
            EventFilter::EVFILT_READ,
            EvFlags::EV_ADD,
            FilterFlag::empty(),
            0,
            0,
        );
        self.readiness = match queue.kevent(&[event], &mut [], None) {
            Ok(_) => Some(queue),
            // Darwin's /dev/tty alias rejects kqueue, unlike a resolved PTY.
            Err(nix::errno::Errno::EINVAL | nix::errno::Errno::ENOTSUP) => None,
            Err(error) => return Err(error.into()),
        };
        if self.readiness.is_none() && self.input.as_raw_fd() as usize >= nix::libc::FD_SETSIZE {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Darwin /dev/tty readiness requires an owned descriptor below FD_SETSIZE",
            ));
        }
        Ok(())
    }
    #[cfg(target_os = "linux")]
    fn wait_readable(&self, timeout: Duration) -> io::Result<bool> {
        let ts = Timespec {
            tv_sec: timeout.as_secs().try_into().unwrap_or(i64::MAX),
            tv_nsec: timeout.subsec_nanos().into(),
        };
        let mut fds = [PollFd::new(&self.input, PollFlags::IN)];
        match poll(&mut fds, Some(&ts)) {
            Ok(n) => Ok(n != 0),
            Err(rustix::io::Errno::INTR) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }
    #[cfg(target_os = "macos")]
    fn wait_readable(&self, timeout: Duration) -> io::Result<bool> {
        let ts = nix::libc::timespec {
            tv_sec: timeout.as_secs().try_into().unwrap_or(i64::MAX),
            tv_nsec: timeout.subsec_nanos().into(),
        };
        let mut events = [Readiness::new(
            0,
            EventFilter::EVFILT_READ,
            EvFlags::empty(),
            FilterFlag::empty(),
            0,
            0,
        )];
        let Some(queue) = &self.readiness else {
            use nix::sys::{
                select::{FdSet, select},
                time::{TimeVal, TimeValLike},
            };
            if self.input.as_raw_fd() as usize >= nix::libc::FD_SETSIZE {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "Darwin /dev/tty readiness requires an owned descriptor below FD_SETSIZE",
                ));
            }
            let mut set = FdSet::new();
            set.insert(self.input.as_fd());
            let mut tv = TimeVal::microseconds(timeout.as_micros().min(i64::MAX as u128) as i64);
            return match select(None, Some(&mut set), None, None, Some(&mut tv)) {
                Ok(n) => Ok(n != 0),
                Err(nix::errno::Errno::EINTR) => Ok(false),
                Err(e) => Err(e.into()),
            };
        };
        match queue.kevent(&[], &mut events, Some(ts)) {
            Ok(n) => Ok(n != 0),
            Err(nix::errno::Errno::EINTR) => Ok(false),
            Err(e) => Err(e.into()),
        }
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
    fn read(&self, buffer: &mut [u8], timeout: Duration) -> io::Result<Read> {
        if !self.wait_readable(timeout)? {
            return Ok(Read::Idle);
        }
        match read(&self.input, buffer) {
            Ok(0) => Ok(Read::Eof),
            Ok(n) => Ok(Read::Bytes(n)),
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
