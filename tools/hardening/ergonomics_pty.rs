//! Standalone real-PTY oracle for interaction-ergonomics memory qualification.

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod native {
    use replai::{Editor, Interaction, Prompt, Role, Theme};
    use rustix::{
        fs::{OFlags, fcntl_getfl, fcntl_setfl},
        io::{FdFlags, fcntl_setfd, read, write},
        pty::{OpenptFlags, grantpt, openpt, unlockpt},
        termios::{Winsize, tcgetattr, tcsetwinsize},
    };
    #[cfg(target_os = "macos")]
    use rustix::fs::{Mode, open};
    use std::{os::fd::OwnedFd, time::Duration};

    fn pair(cols: u16, rows: u16) -> (OwnedFd, OwnedFd) {
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
        tcsetwinsize(
            &slave,
            Winsize {
                ws_col: cols,
                ws_row: rows,
                ws_xpixel: 0,
                ws_ypixel: 0,
            },
        )
        .unwrap();
        fcntl_setfl(&master, fcntl_getfl(&master).unwrap() | OFlags::NONBLOCK).unwrap();
        (master, slave)
    }

    fn drain(master: &OwnedFd) -> Vec<u8> {
        let mut out = Vec::new();
        let mut buf = [0; 8192];
        loop {
            match read(master, &mut buf) {
                Ok(0) | Err(rustix::io::Errno::AGAIN | rustix::io::Errno::IO) => return out,
                Ok(n) => out.extend_from_slice(&buf[..n]),
                Err(error) => panic!("PTY read failed: {error}"),
            }
        }
    }

    fn feed(
        interaction: &mut Interaction,
        master: &OwnedFd,
        bytes: &[u8],
        screen: &mut vt100::Parser,
    ) {
        for byte in bytes {
            assert_eq!(write(master, &[*byte]).unwrap(), 1);
            interaction.poll(Duration::from_millis(20)).unwrap();
            screen.process(&drain(master));
        }
    }

    pub fn run() {
        for (cols, plain) in [(20, true), (40, false), (80, true), (132, false)] {
            let (master, slave) = pair(cols, 12);
            let before = format!("{:?}", tcgetattr(&slave).unwrap());
            let mut editor = Editor::new(256, 4);
            editor.admit_history("older alpha command").unwrap();
            editor.admit_history("newest beta command").unwrap();
            editor.insert("draft界").unwrap();
            editor.left();
            let original = editor.analysis_snapshot();
            let mut interaction = Interaction::new(editor);
            interaction
                .open_with_theme(
                    &slave,
                    &slave,
                    Prompt::new("demo").unwrap(),
                    Theme::new(true, plain, None),
                )
                .unwrap();
            let mut screen = vt100::Parser::new(12, cols, 100);
            screen.process(&drain(&master));

            feed(&mut interaction, &master, b"\x12alpha", &mut screen);
            assert_eq!(interaction.history_search_query(), Some("alpha"));
            assert_eq!(interaction.history_search_match(), Some("older alpha command"));
            assert_eq!(interaction.editor().text(), original.text());
            assert_eq!(interaction.editor().cursor(), original.cursor());
            assert_eq!(interaction.revision(), original.revision());
            assert!(screen.screen().contents().contains("? 'alpha' >"));

            interaction.external_output(Role::Dim, "host notice").unwrap();
            screen.process(&drain(&master));
            assert_eq!(interaction.history_search_query(), Some("alpha"));
            feed(&mut interaction, &master, b"\x1b", &mut screen);
            std::thread::sleep(Duration::from_millis(270));
            interaction.poll(Duration::ZERO).unwrap();
            screen.process(&drain(&master));
            assert_eq!(interaction.editor().text(), original.text());
            assert_eq!(interaction.editor().cursor(), original.cursor());
            assert_eq!(interaction.revision(), original.revision());

            feed(&mut interaction, &master, b"\x12alpha\r", &mut screen);
            assert_eq!(interaction.editor().text(), "older alpha command");
            assert_ne!(interaction.revision(), original.revision());
            feed(&mut interaction, &master, b"\x1f", &mut screen);
            assert_eq!(
                (interaction.editor().text(), interaction.editor().cursor()),
                ("draft界", "draft".len())
            );
            feed(&mut interaction, &master, b"\x1bf\x17\x19", &mut screen);
            assert_eq!(interaction.editor().text(), "draft界");
            interaction.close().unwrap();
            screen.process(&drain(&master));
            assert_eq!(format!("{:?}", tcgetattr(&slave).unwrap()), before);
            assert!(!screen.screen().bracketed_paste());
        }
        println!("{{\"plain_styled\":true,\"real_pty\":true,\"widths\":[20,40,80,132]}}");
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn main() {
    native::run();
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn main() {
    eprintln!("ergonomics-pty requires Linux or macOS");
    std::process::exit(2);
}
