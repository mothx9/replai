//! Native PTY campaign host. Every interaction call uses the public surface.
#[cfg(unix)]
#[path = "../../tests/support/posix_pty.rs"]
mod pty;
#[cfg(unix)]
mod posix {
    use super::pty;
    use replai::*;
    use rustix::{
        io::{read, write},
        termios::{Winsize, tcgetattr, tcsetwinsize},
    };
    use std::{
        env, fs,
        os::fd::{AsFd, OwnedFd},
        time::Duration,
    };
    fn size(fd: &impl AsFd, w: usize, h: usize) {
        tcsetwinsize(
            fd,
            Winsize {
                ws_row: h as u16,
                ws_col: w as u16,
                ws_xpixel: 0,
                ws_ypixel: 0,
            },
        )
        .unwrap();
    }
    fn drain(fd: &OwnedFd) -> Vec<u8> {
        let mut output = Vec::new();
        let mut bytes = [0; 32768];
        while let Ok(n) = read(fd, &mut bytes) {
            if n == 0 {
                break;
            }
            output.extend_from_slice(&bytes[..n]);
        }
        output
    }
    fn ready(t: &mut Interaction, master: &OwnedFd, bytes: &[u8], driven: bool) -> Option<Event> {
        assert_eq!(write(master, bytes).unwrap(), bytes.len());
        // The host owns the actual native wait; REPLAI gets only its result.
        let fd = t.input_source().unwrap();
        let mut fds = [rustix::event::PollFd::new(
            &fd,
            rustix::event::PollFlags::IN,
        )];
        assert!(
            rustix::event::poll(
                &mut fds,
                Some(&rustix::event::Timespec {
                    tv_sec: 1,
                    tv_nsec: 0
                })
            )
            .unwrap()
                > 0
        );
        for _ in 0..4096 {
            let event = if driven {
                t.advance(Wake::InputReady).unwrap()
            } else {
                t.poll(Duration::ZERO).unwrap()
            };
            if event.is_some() || !t.is_open() {
                return event;
            }
            if t.wait_interest().unwrap() != WaitInterest::Ready {
                return None;
            }
        }
        panic!("bounded input did not drain");
    }
    fn count_fds() -> usize {
        fs::read_dir(if cfg!(target_os = "linux") {
            "/proc/self/fd"
        } else {
            "/dev/fd"
        })
        .unwrap()
        .count()
    }
    fn config(plain: bool, paste: bool) -> TerminalConfig {
        TerminalConfig {
            facts: TerminalFacts::assumed_vt(),
            styling: if plain {
                FeaturePolicy::Disabled
            } else {
                FeaturePolicy::Preferred
            },
            bracketed_paste: if paste {
                FeaturePolicy::Preferred
            } else {
                FeaturePolicy::Disabled
            },
            theme: Theme::new(true, plain, None),
        }
    }
    fn failures(repeats: usize, exhaustion: bool) {
        let before_fds = count_fds();
        let mut restorable_hangups = 0;
        let mut unrestorable_hangups = 0;
        if exhaustion {
            // This binary is an isolated child. Never change its controller's limit.
            rustix::process::setrlimit(
                rustix::process::Resource::Nofile,
                rustix::process::Rlimit {
                    current: Some(64),
                    maximum: Some(64),
                },
            )
            .unwrap();
        }
        let classes = if exhaustion {
            vec!["descriptor-exhaustion"]
        } else {
            vec![
                "admission",
                "geometry-acquire",
                "write-acquire",
                "geometry-active",
                "read-hangup",
                "write-hangup",
                "capacity",
                "host-payload",
            ]
        };
        for class in &classes {
            for _ in 0..repeats {
                let (master, slave) = pty::pair();
                size(&slave, 40, 10);
                rustix::fs::fcntl_setfl(&master, rustix::fs::OFlags::NONBLOCK).unwrap();
                let original = format!("{:?}", tcgetattr(&slave).unwrap());
                let mut t = Interaction::new(Editor::new(256, 8));
                match *class {
                    "admission" => {
                        let mut c = config(true, true);
                        c.facts.cursor = FeatureSupport::Unknown;
                        assert!(
                            t.open_with_config(&slave, &slave, Prompt::new("fail").unwrap(), c)
                                .is_err()
                        );
                        assert!(!t.is_open());
                    }
                    "geometry-acquire" => {
                        size(&slave, 0, 0);
                        assert!(
                            t.open_with_config(
                                &slave,
                                &slave,
                                Prompt::new("fail").unwrap(),
                                config(true, true)
                            )
                            .is_err()
                        );
                        assert!(!t.is_open());
                    }
                    "write-acquire" => {
                        let name = rustix::pty::ptsname(&master, Vec::new()).unwrap();
                        let readonly = rustix::fs::open(
                            name,
                            rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::NOCTTY,
                            rustix::fs::Mode::empty(),
                        )
                        .unwrap();
                        assert!(
                            t.open_with_config(
                                &slave,
                                &readonly,
                                Prompt::new("fail").unwrap(),
                                config(true, true)
                            )
                            .is_err()
                        );
                        assert!(!t.is_open());
                    }
                    "descriptor-exhaustion" => {
                        let mut held = Vec::new();
                        while let Ok(f) = fs::File::open("/dev/null") {
                            held.push(f);
                        }
                        assert!(
                            t.open_with_config(
                                &slave,
                                &slave,
                                Prompt::new("fail").unwrap(),
                                config(true, true)
                            )
                            .is_err()
                        );
                        assert!(!t.is_open());
                        drop(held);
                        t.open_with_config(
                            &slave,
                            &slave,
                            Prompt::new("retry").unwrap(),
                            config(true, true),
                        )
                        .unwrap();
                        let mut held = Vec::new();
                        while let Ok(f) = fs::File::open("/dev/null") {
                            held.push(f);
                        }
                        t.close().unwrap();
                        drop(held);
                    }
                    _ => {
                        t.open_with_config(
                            &slave,
                            &slave,
                            Prompt::new("fail").unwrap(),
                            config(true, true),
                        )
                        .unwrap();
                        drain(&master);
                        if class.ends_with("hangup") {
                            drop(master);
                            let result = if *class == "read-hangup" {
                                t.advance(Wake::InputReady)
                                    .map(|event| assert_eq!(event, Some(Event::EndOfInput)))
                            } else {
                                t.external_output(Role::Default, "after disconnect")
                            };
                            // Darwin may still permit termios restoration after
                            // hangup. Linux can refuse it. Qualify the OS fact,
                            // never prescribe Linux cleanup outcomes to macOS.
                            let _ = t.close();
                            assert!(!t.is_open());
                            if let Ok(after) = tcgetattr(&slave) {
                                assert_eq!(format!("{after:?}"), original);
                                restorable_hangups += 1;
                            } else {
                                assert!(result.is_err(), "unrestorable PTY must report failure");
                                unrestorable_hangups += 1;
                            }
                            drop(t);
                            drop(slave);
                            assert_eq!(count_fds(), before_fds);
                            continue;
                        }
                        if *class == "geometry-active" {
                            size(&slave, 0, 0);
                            assert!(t.advance(Wake::Resize).is_err());
                            assert!(!t.is_open());
                        } else {
                            let before = t.analysis_snapshot();
                            if *class == "capacity" {
                                assert!(t.complete(0..0, &"x".repeat(257)).is_err());
                            } else {
                                assert!(t.complete(0..0, "\x1b[2J").is_err());
                                assert!(
                                    t.external_output(Role::Default, "\x1b]52;bad\x07").is_err()
                                );
                            }
                            assert_eq!(t.revision(), before.revision());
                            assert_eq!(t.editor().text(), before.text());
                            assert!(drain(&master).is_empty());
                        }
                    }
                }
                t.close().unwrap();
                assert_eq!(format!("{:?}", tcgetattr(&slave).unwrap()), original);
                drop(t);
                drop(master);
                drop(slave);
                assert_eq!(count_fds(), before_fds);
            }
        }
        println!(
            "{}",
            serde_json::json!({"native_failure_classes":classes,"repetitions":repeats,"fd_stable":true,"restorable_hangups":restorable_hangups,"unrestorable_hangups":unrestorable_hangups,"restoration":"exact when possible; explicit failure when OS refuses"})
        );
    }
    pub fn main() {
        let args: Vec<_> = env::args().collect();
        let count: usize = args[2].parse().unwrap();
        assert!((1..=100_000).contains(&count));
        if args[1] == "failures" || args[1] == "exhaustion" {
            failures(count, args[1] == "exhaustion");
            return;
        }
        if args[1] == "blocking" {
            let before = format!("{:?}", tcgetattr(std::io::stdin()).unwrap());
            let mut t = Interaction::new(Editor::new(4096, 8));
            let fds = count_fds();
            for i in 0..count {
                t.editor_mut().unwrap().clear();
                let result = t.read_line(Prompt::new("block").unwrap()).unwrap();
                assert_eq!(result, ReadOutcome::Submitted(format!("case{i} 界\nline")));
                assert_eq!(
                    format!("{:?}", tcgetattr(std::io::stdin()).unwrap()),
                    before
                );
                assert_eq!(count_fds(), fds);
                println!("RECEIPT {i}");
            }
            println!("{{\"tier\":\"blocking\",\"cycles\":{count},\"restoration\":true}}");
            return;
        }
        if args[1] == "cabi" {
            let fds = count_fds();
            for _ in 0..count {
                replai_hardening::cabi_case(b">abc<abcDabc=abc");
                assert_eq!(count_fds(), fds);
            }
            println!("{{\"tier\":\"cabi\",\"cycles\":{count},\"fd_stable\":true}}");
            return;
        }
        let driven = args[1] != "session";
        let mixed = args[1] == "mixed";
        let cycles = if mixed { count.div_ceil(20) } else { count };
        let fds = count_fds();
        let mut events = 0;
        for cycle in 0..cycles {
            let (master, slave) = pty::pair();
            rustix::fs::fcntl_setfl(&master, rustix::fs::OFlags::NONBLOCK).unwrap();
            let width = [20, 40, 80, 132][cycle % 4];
            let height = 6 + cycle % 20;
            size(&slave, width, height);
            let original = format!("{:?}", tcgetattr(&slave).unwrap());
            let mut t = Interaction::new(Editor::new(65536, 8));
            t.editor_mut()
                .unwrap()
                .admit_history("history\nentry")
                .unwrap();
            t.set_submission_policy(SubmissionPolicy::Validated)
                .unwrap();
            let plain = cycle % 2 == 0;
            let paste = cycle % 3 != 0;
            if driven {
                t.open_with_config(
                    &slave,
                    &slave,
                    Prompt::new("native").unwrap(),
                    config(plain, paste),
                )
                .unwrap();
            } else {
                t.open_with_theme(
                    &slave,
                    &slave,
                    Prompt::new("native").unwrap(),
                    Theme::new(true, plain, None),
                )
                .unwrap();
            }
            drain(&master);
            let payload = if t.features().unwrap().bracketed_paste {
                "\x1b[200~e\u{301}界\nline\x1b[201~"
            } else {
                "e\u{301}界"
            };
            assert!(ready(&mut t, &master, payload.as_bytes(), driven).is_none());
            events += 1;
            let stale = t.analysis_snapshot();
            ready(&mut t, &master, b"x", driven);
            events += 1;
            drain(&master);
            let candidates = CompletionSet::new(
                stale.revision(),
                vec![CompletionCandidate::new(0..0, "bad", "bad").unwrap()],
            )
            .unwrap();
            assert_eq!(
                t.present_completions(candidates).unwrap(),
                AnalysisOutcome::Stale
            );
            events += 1;
            assert!(drain(&master).is_empty());
            assert_eq!(
                t.present_analysis(
                    AnalysisPresentation::new(
                        stale.revision(),
                        vec![],
                        Some(Hint::new("stale", Role::Dim).unwrap())
                    )
                    .unwrap()
                )
                .unwrap(),
                AnalysisOutcome::Stale
            );
            events += 1;
            assert!(drain(&master).is_empty());
            assert_eq!(
                t.apply_validation(
                    ValidationResult::new(stale.revision(), ValidationDisposition::Complete)
                        .unwrap()
                )
                .unwrap()
                .analysis,
                AnalysisOutcome::Stale
            );
            events += 1;
            assert!(drain(&master).is_empty());
            let current = t.analysis_snapshot();
            t.present_analysis(
                AnalysisPresentation::new(
                    current.revision(),
                    vec![AnalysisSpan::new(0..current.text().len(), Role::Accent).unwrap()],
                    Some(Hint::new("hint", Role::Dim).unwrap()),
                )
                .unwrap(),
            )
            .unwrap();
            events += 1;
            drain(&master);
            t.present_completions(
                CompletionSet::new(
                    current.revision(),
                    vec![
                        CompletionCandidate::new(
                            0..current.text().len(),
                            "accepted\nvalue",
                            "accept",
                        )
                        .unwrap(),
                        CompletionCandidate::new(0..current.text().len(), "other", "other")
                            .unwrap(),
                    ],
                )
                .unwrap(),
            )
            .unwrap();
            events += 1;
            drain(&master);
            ready(&mut t, &master, b"\t", driven);
            events += 1;
            assert_eq!(t.completion_selection().unwrap().index, 1);
            assert_eq!(t.revision(), current.revision());
            drain(&master);
            let selection = t.completion_selection();
            t.external_output(Role::Success, "host notice").unwrap();
            events += 1;
            drain(&master);
            size(&slave, [132, 80, 40, 20][cycle % 4], height + 1);
            if driven {
                t.advance(Wake::Resize).unwrap();
            } else {
                t.poll(Duration::ZERO).unwrap();
            }
            events += 1;
            drain(&master);
            assert_eq!(t.completion_selection(), selection);
            assert_eq!(t.revision(), current.revision());
            ready(&mut t, &master, b"\x1b[Z", driven);
            events += 1;
            drain(&master);
            assert!(ready(&mut t, &master, b"\r", driven).is_none());
            events += 1;
            drain(&master);
            assert_eq!(t.editor().text(), "accepted\nvalue");
            assert!(t.analysis_presentation().is_none());
            assert!(matches!(
                ready(&mut t, &master, b"\r", driven),
                Some(Event::SubmissionRequested(_))
            ));
            events += 1;
            drain(&master);
            let r = t.revision();
            t.apply_validation(
                ValidationResult::new(
                    r,
                    ValidationDisposition::Invalid(vec![
                        Diagnostic::new("Correct the draft", Some(0..8)).unwrap(),
                    ]),
                )
                .unwrap(),
            )
            .unwrap();
            events += 1;
            drain(&master);
            t.external_output(Role::Default, "another notice").unwrap();
            events += 1;
            drain(&master);
            assert_eq!(t.revision(), r);
            assert!(t.diagnostics().is_some());
            ready(&mut t, &master, b"\x1b[A", driven);
            events += 1;
            drain(&master);
            assert_ne!(t.revision(), r);
            assert!(t.diagnostics().is_none());
            ready(&mut t, &master, b"\r", driven);
            events += 1;
            drain(&master);
            t.apply_validation(
                ValidationResult::new(t.revision(), ValidationDisposition::Incomplete).unwrap(),
            )
            .unwrap();
            events += 1;
            drain(&master);
            ready(&mut t, &master, b"\r", driven);
            events += 1;
            drain(&master);
            let submitted = t.editor().text().to_owned();
            let result = t
                .apply_validation(
                    ValidationResult::new(t.revision(), ValidationDisposition::Complete).unwrap(),
                )
                .unwrap();
            events += 1;
            assert_eq!(result.event, Some(Event::Submitted(submitted)));
            t.close().unwrap();
            assert_eq!(format!("{:?}", tcgetattr(&slave).unwrap()), original);
            let mut screen = vt100::Parser::new(30, 132, 0);
            screen.process(&drain(&master));
            assert!(!screen.screen().bracketed_paste());
            drop(t);
            drop(master);
            drop(slave);
            assert_eq!(count_fds(), fds);
        }
        println!(
            "{{\"tier\":\"{}\",\"cycles\":{cycles},\"events\":{events},\"fd_stable\":true,\"restoration\":true}}",
            args[1]
        );
    }
}
fn main() {
    #[cfg(unix)]
    posix::main();
    #[cfg(not(unix))]
    panic!("native terminal campaign is Linux/macOS only");
}
