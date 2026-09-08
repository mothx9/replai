// Real native PTY fixture. Only slave acquisition differs between OS contracts.
#[cfg(target_os = "macos")]
use rustix::fs::{Mode, OFlags, open};
use rustix::{
    io::{FdFlags, fcntl_setfd},
    pty::{OpenptFlags, grantpt, openpt, unlockpt},
};
use std::os::fd::OwnedFd;

pub fn pair() -> (OwnedFd, OwnedFd) {
    let master = openpt(OpenptFlags::RDWR | OpenptFlags::NOCTTY).unwrap();
    fcntl_setfd(&master, FdFlags::CLOEXEC).unwrap();
    grantpt(&master).unwrap();
    unlockpt(&master).unwrap();
    #[cfg(target_os = "linux")]
    let slave = rustix::pty::ioctl_tiocgptpeer(
        &master,
        OpenptFlags::RDWR | OpenptFlags::NOCTTY | OpenptFlags::CLOEXEC,
    )
    .unwrap();
    #[cfg(target_os = "macos")]
    let slave = open(
        rustix::pty::ptsname(&master, Vec::new()).unwrap(),
        OFlags::RDWR | OFlags::NOCTTY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .unwrap();
    (master, slave)
}
