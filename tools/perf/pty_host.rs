//! POSIX production-facade host. Control receipts use stderr, never terminal output.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[path = "exit_gate.rs"]
mod exit_gate;
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn main() {
    let _exit_gate = exit_gate::Gate::new();
    use replai::{Editor, Event, Interaction, Prompt, Role};
    use std::{
        io::{self},
        time::{Duration, Instant},
    };
    let args: Vec<_> = std::env::args().collect();
    let arg = |key: &str, default: &str| {
        args.windows(2)
            .find(|a| a[0] == key)
            .map(|a| a[1].clone())
            .unwrap_or(default.into())
    };
    let case_path = arg("--case-file", "");
    let case: serde_json::Value = if case_path.is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_slice(&std::fs::read(case_path).unwrap()).unwrap()
    };
    let mode = arg("--mode", "edit");
    let initial = case["initial"]
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| arg("--initial", ""));
    let limit: usize = arg("--limit", "1048576").parse().unwrap();
    let timeout: u64 = arg("--timeout-ms", "100").parse().unwrap();
    let mut e = Editor::new(limit, 100);
    for s in ["history first", "history second"] {
        e.admit_history(s).unwrap();
    }
    e.insert(&initial).unwrap();
    if let Some(pos) = case["cursor"].as_u64() {
        e.replace(pos as usize..pos as usize, "").unwrap();
    }
    if args.iter().any(|s| s == "--middle") {
        e.home();
        for _ in 0..initial.chars().count() / 2 {
            e.right();
        }
    }
    let mut i = Interaction::new(e);
    if case["controlling_tty"].as_bool().unwrap_or(false) {
        let tty = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .unwrap();
        i.open(&tty, &tty, Prompt::new("p").unwrap()).unwrap();
        // The duplicated resource must survive closing the caller's descriptors.
        drop(tty);
    } else {
        i.open(&io::stdin(), &io::stdout(), Prompt::new("p").unwrap())
            .unwrap();
    }
    eprintln!("READY");
    if mode == "idle" || mode == "output" {
        use std::io::Read;
        let mut gate = [0];
        io::stdin().read_exact(&mut gate).unwrap();
        assert_eq!(gate, [b'!']);
    }
    if mode == "idle" {
        let seconds: f64 = arg("--seconds", "3").parse().unwrap();
        let t = Instant::now();
        let mut polls = 0;
        while t.elapsed().as_secs_f64() < seconds {
            assert!(i.poll(Duration::from_millis(timeout)).unwrap().is_none());
            polls += 1;
        }
        let seconds = t.elapsed().as_secs_f64();
        i.close().unwrap();
        eprintln!(
            "{}",
            serde_json::json!({"poll_calls":polls,"elapsed_seconds":seconds,"timeout_ms":timeout})
        );
        return;
    }
    if mode == "output" {
        let rate: f64 = arg("--rate", "20").parse().unwrap();
        let seconds: f64 = arg("--seconds", "2").parse().unwrap();
        let size: usize = arg("--size", "80").parse().unwrap();
        let unit = "host output line\n";
        let mut chunk = unit.repeat(size / unit.len());
        chunk.push_str(&"x".repeat(size - chunk.len()));
        let before = (i.editor().text().to_string(), i.editor().cursor());
        let start = Instant::now();
        let calls = (seconds * rate).ceil() as usize;
        let mut samples = Vec::with_capacity(calls);
        let mut late = 0;
        for n in 0..calls {
            // Scheduling is outside the host-call timer; no editing, worker or queue.
            let due = Duration::from_secs_f64(n as f64 / rate);
            if let Some(wait) = due.checked_sub(start.elapsed()) {
                std::thread::sleep(wait);
            } else if n > 0 {
                late += 1;
            }
            let t = Instant::now();
            i.external_output(Role::Dim, &chunk).unwrap();
            samples.push(t.elapsed().as_secs_f64() * 1e6);
            assert_eq!(
                (i.editor().text(), i.editor().cursor()),
                (before.0.as_str(), before.1)
            );
        }
        let elapsed = start.elapsed().as_secs_f64();
        i.close().unwrap();
        eprintln!(
            "{}",
            serde_json::json!({"samples_us":samples,"calls":calls,"rate":rate,"input_bytes":size,"elapsed_seconds":elapsed,"missed_deadlines":late,"draft_restored":true})
        );
        return;
    }
    // Receipts only follow a completed production poll/complete write. They add
    // IPC/scheduling to the external end-to-end bound, never an artificial delay.
    let mut polls = 0;
    loop {
        let event = i.poll(Duration::from_millis(timeout)).unwrap();
        polls += 1;
        match event {
            Some(Event::CompletionRequested) => {
                let replacement = case["replacement"]
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| arg("--replacement", "replacement"));
                let n = i.editor().text().len();
                let t = Instant::now();
                i.complete(0..n, &replacement).unwrap();
                eprintln!(
                    "{}",
                    serde_json::json!({"complete_us":t.elapsed().as_secs_f64()*1e6})
                );
            }
            Some(Event::Submitted(text)) => {
                if mode == "comparison" {
                    eprintln!(
                        "SUBMITTED:{}",
                        text.as_bytes()
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect::<String>()
                    );
                } else {
                    eprintln!(
                        "{}",
                        serde_json::json!({"submitted":text,"poll_calls":polls})
                    );
                }
                break;
            }
            Some(Event::Rejected(error)) => {
                eprintln!("{}", serde_json::json!({"rejected":error.to_string()}));
            }
            Some(Event::Interrupted | Event::EndOfInput) => break,
            Some(Event::SubmissionRequested(_)) => unreachable!("direct benchmark"),
            None => {}
        }
    }
    i.close().unwrap();
}
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn main() {
    eprintln!("UNSUPPORTED: the real system backend requires Linux or macOS");
}
