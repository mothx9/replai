//! An embeddable terminal interaction library for line-oriented and REPL-style
//! command interfaces.
//!
//! Platform-neutral editing and interaction engine, with a shared Linux/macOS POSIX system façade.
//! Hosts retain input meaning, completion discovery and history admission.
//!
//! ```
//! use replai::Editor;
//! let mut draft = Editor::new(1024, 20);
//! draft.insert("café 界")?;
//! draft.left();
//! draft.delete();
//! assert_eq!(draft.text(), "café ");
//! # Ok::<(), replai::EditError>(())
//! ```
//!
//! A host owns the loop, including what to do after submission or interruption:
//!
//! ```no_run
//! # #[cfg(any(target_os = "linux", target_os = "macos"))]
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use replai::{Editor, Event, Prompt, Interaction};
//! use std::time::Duration;
//! let mut terminal = Interaction::new(Editor::new(65_536, 100));
//! terminal.open(
//!     &std::io::stdin(), &std::io::stdout(), Prompt::new("demo")?,
//! )?;
//! loop {
//!     match terminal.poll(Duration::from_millis(100))? {
//!         Some(Event::Submitted(text)) => { /* host consumes text */ break; }
//!         Some(Event::Interrupted | Event::EndOfInput) => break,
//!         Some(Event::CompletionRequested) => { /* host may call complete */ }
//!         Some(Event::Rejected(error)) => { /* host may report error */ }
//!         None => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```

mod capabilities;
mod core;
mod event;
mod interaction;
// Other system façades are intentionally absent. These internal components
// compile everywhere and execute through deterministic tests on every CI OS.
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
mod actions;
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
mod engine;
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
mod input;
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
mod keymap;
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
mod presentation;
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
mod protocol;
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
mod render;
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
mod substrate;
#[cfg(any(target_os = "linux", target_os = "macos"))]
mod system;
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
mod terminal;
pub use core::{EditError, Editor};
pub use event::{Error, Event};
pub use interaction::Interaction;
pub use presentation::{Prompt, Role, Theme};

#[cfg(test)]
mod conformance;
