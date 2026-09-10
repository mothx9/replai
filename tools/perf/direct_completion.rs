//! Public I0 replacement control, copied unchanged into exact-source archive builds.
#[path = "allocation.rs"]
mod allocation;
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[path = "../../tests/support/posix_pty.rs"]
mod pty_support;
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn main() {
    use replai::{AnalysisOutcome, Editor, Interaction, Prompt};
    use rustix::{
        fs::{OFlags, fcntl_getfl, fcntl_setfl},
        io::read,
        termios::{Winsize, tcgetattr, tcsetwinsize},
    };
    use std::{hint::black_box, time::Instant};
    for native in [false, true] {
        let mut times = Vec::new();
        let mut memories = Vec::new();
        let mut bytes = Vec::new();
        for i in 0..65 {
            let mut e = Editor::new(65_536, 0);
            e.insert(&"a".repeat(1024)).unwrap();
            let revision = e.revision();
            if native {
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
                fcntl_setfl(&master, fcntl_getfl(&master).unwrap() | OFlags::NONBLOCK).unwrap();
                let saved = tcgetattr(&slave).unwrap();
                let mut interaction = Interaction::new(e);
                interaction
                    .open(&slave, &slave, Prompt::new("p").unwrap())
                    .unwrap();
                let mut buffer = [0; 8192];
                while read(&master, &mut buffer).is_ok() {}
                let memory = allocation::start();
                let start = Instant::now();
                assert_eq!(
                    black_box(interaction.complete_at(revision, 0..1024, "build").unwrap()),
                    AnalysisOutcome::Applied
                );
                let elapsed = start.elapsed().as_secs_f64() * 1e6;
                let memory = allocation::end(memory);
                assert_eq!(interaction.editor().text(), "build");
                let mut count = 0;
                while let Ok(n) = read(&master, &mut buffer) {
                    if n == 0 {
                        break;
                    }
                    count += n;
                }
                interaction.close().unwrap();
                assert_eq!(
                    format!("{:?}", tcgetattr(&slave).unwrap()),
                    format!("{saved:?}")
                );
                if i >= 2 {
                    times.push(elapsed);
                    memories.push(memory);
                    bytes.push(count);
                }
            } else {
                let memory = allocation::start();
                let start = Instant::now();
                assert_eq!(
                    black_box(e.replace_at(revision, 0..1024, "build").unwrap()),
                    AnalysisOutcome::Applied
                );
                let elapsed = start.elapsed().as_secs_f64() * 1e6;
                let memory = allocation::end(memory);
                assert_eq!(e.text(), "build");
                if i >= 2 {
                    times.push(elapsed);
                    memories.push(memory);
                }
            }
        }
        println!(
            "{}",
            serde_json::json!({"operation":if native {"Interaction::complete_at"} else {"Editor::replace_at"},"input_bytes":1024,"replacement":"build","samples_us":if cfg!(feature="allocations"){None}else{Some(times)},"allocation_samples":if cfg!(feature="allocations"){Some(memories)}else{None},"terminal_bytes":bytes})
        );
    }
}
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn main() {
    eprintln!("Native direct replacement control requires Linux/macOS");
}
