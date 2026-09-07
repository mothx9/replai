//! Internal byte-transport conformance contract, not the future public driver API.
use std::{io, time::Duration};

pub(crate) enum Read {
    Byte(u8),
    Eof,
    Idle,
}
pub(crate) trait Transport {
    fn dimensions(&self) -> io::Result<(usize, usize)>;
    fn read(&self, timeout: Duration) -> io::Result<Read>;
    fn write(&self, bytes: &[u8]) -> io::Result<()>;
    /// Best-effort protocol cleanup, with a backend-specific alternate route if available.
    fn cleanup_write(&self, bytes: &[u8]) -> io::Result<()>;
    /// Restore precisely the captured resource mode; retryable and idempotent.
    fn restore(&mut self) -> io::Result<()>;
}

pub(crate) fn combine(first: io::Result<()>, second: io::Result<()>) -> io::Result<()> {
    match (first, second) {
        (Err(a), Err(b)) => Err(io::Error::new(
            a.kind(),
            format!("{a}; terminal cleanup also failed: {b}"),
        )),
        (Err(e), _) | (_, Err(e)) => Err(e),
        _ => Ok(()),
    }
}
