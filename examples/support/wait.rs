//! Host-owned POSIX wait, deliberately outside REPLAI's interaction implementation.
use std::{io, os::fd::BorrowedFd, time::Duration};

pub fn wait(sources: &[BorrowedFd<'_>], timeout: Option<Duration>) -> io::Result<Vec<bool>> {
    #[cfg(target_os = "linux")]
    {
        use rustix::event::{PollFd, PollFlags, Timespec, poll};
        let mut fds: Vec<_> = sources
            .iter()
            .map(|fd| PollFd::new(fd, PollFlags::IN))
            .collect();
        let timeout = timeout
            .map(Timespec::try_from)
            .transpose()
            .map_err(io::Error::other)?;
        match poll(&mut fds, timeout.as_ref()) {
            Ok(_) => Ok(fds.iter().map(|fd| !fd.revents().is_empty()).collect()),
            Err(rustix::io::Errno::INTR) => Ok(vec![false; fds.len()]),
            Err(e) => Err(e.into()),
        }
    }
    #[cfg(target_os = "macos")]
    {
        use nix::poll::{PollFd, PollFlags, PollTimeout, poll};
        let mut fds: Vec<_> = sources
            .iter()
            .map(|fd| PollFd::new(*fd, PollFlags::POLLIN))
            .collect();
        // Round upward: millisecond POSIX poll must not spin before a deadline.
        let timeout = match timeout {
            None => PollTimeout::NONE,
            Some(d) => PollTimeout::try_from(
                i32::try_from(d.as_millis() + u128::from(d.subsec_nanos() % 1_000_000 != 0))
                    .unwrap_or(i32::MAX),
            )
            .map_err(io::Error::other)?,
        };
        match poll(&mut fds, timeout) {
            Ok(_) => Ok(fds
                .iter()
                .map(|fd| fd.revents().is_some_and(|v| !v.is_empty()))
                .collect()),
            Err(nix::errno::Errno::EINTR) => Ok(vec![false; fds.len()]),
            Err(e) => Err(io::Error::from_raw_os_error(e as i32)),
        }
    }
}
