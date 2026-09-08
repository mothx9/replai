// Benchmark-only lifecycle barrier. Timers stop before this handshake.
// Darwin revokes a controlling PTY when its session leader exits; the parent
// checks restored modes while the child is still alive, after normal drops.
use std::{
    fs::File,
    io::{Read, Write},
};
pub struct Gate(Option<File>);
impl Gate {
    pub fn new() -> Self {
        Self(
            std::env::var("P0_EXIT_FD")
                .ok()
                .map(|fd| File::open(format!("/dev/fd/{fd}")).unwrap()),
        )
    }
}
impl Drop for Gate {
    fn drop(&mut self) {
        if let Some(gate) = &mut self.0 {
            if let Ok(fd) = std::env::var("P0_RECEIPT_FD") {
                let mut out = std::fs::OpenOptions::new()
                    .write(true)
                    .open(format!("/dev/fd/{fd}"))
                    .unwrap();
                writeln!(out, "EXIT_READY").unwrap();
            } else {
                eprintln!("EXIT_READY");
            }
            let mut byte = [0];
            gate.read_exact(&mut byte).unwrap();
            assert_eq!(byte, [b'!']);
        }
    }
}
