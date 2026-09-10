//! An embeddable terminal interaction library for line-oriented and REPL-style
//! command interfaces.
//!
//! Platform-neutral editing and interaction engine, with a shared Linux/macOS POSIX system façade.
//! Hosts retain input meaning, completion discovery and history admission.
//! One [`Interaction`] offers `Interaction::read_line` on Linux/macOS for tiny
//! blocking hosts, compatibility polling, and host-owned readiness/deadline
//! driving. [`WaitInterest`], [`Wake`] and [`Deadline`] are portable scheduling
//! values; resource methods exist only for qualified system backends.
//! No threads, signal handlers or async runtime are installed.
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
//! Retain a coherent draft for host-owned analysis while editing continues:
//!
//! ```
//! use replai::{AnalysisOutcome, Editor};
//! let mut editor = Editor::new(1024, 20);
//! editor.insert("run ta")?;
//! let snapshot = editor.analysis_snapshot();
//! // The host may parse snapshot once and share it with several derivations.
//! editor.insert("sk")?;
//! assert_eq!(editor.replace_at(snapshot.revision(), 4..6, "task")?,
//!            AnalysisOutcome::Stale);
//! assert_eq!(snapshot.text(), "run ta");
//! # Ok::<(), replai::EditError>(())
//! ```
//!
//! Structured information can also be rendered without an active terminal:
//!
//! ```
//! use replai::{Block, Document, Text, Theme};
//! let document = Document::new(vec![Block::Heading {
//!     level: 1, text: Text::new("Connection")?,
//! }])?;
//! assert_eq!(document.render(40, Theme::new(false, false, None))?, "# Connection\n");
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
//!         Some(Event::SubmissionRequested(_)) => { /* opt-in host validation */ }
//!         None => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```

mod validation;
pub use validation::{
    Diagnostic, MAX_DIAGNOSTIC_BYTES, MAX_DIAGNOSTICS, MAX_VALIDATION_BYTES, SubmissionPolicy,
    ValidationDisposition, ValidationError, ValidationOutcome, ValidationResult,
};
mod completion;
pub use completion::{
    CompletionAction, CompletionCandidate, CompletionError, CompletionSelection, CompletionSet,
    MAX_COMPLETION_BYTES, MAX_COMPLETION_CANDIDATES, MAX_COMPLETION_FIELD_BYTES,
};
mod analysis_presentation;
pub use analysis_presentation::{
    AnalysisPresentation, AnalysisPresentationError, AnalysisSpan, Hint, MAX_ANALYSIS_SPANS,
    MAX_HINT_BYTES,
};
mod analysis;
pub use analysis::{AnalysisOutcome, AnalysisSnapshot, DraftRevision};
mod capabilities;
mod width;
pub use width::WidthPolicy;
mod document;
mod driving;
pub use document::{Alignment, Block, Column, Document, ListItem, Severity, Span, Text};
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
pub use presentation::{Foreground, Prompt, Role, Style, Theme};

#[cfg(test)]
mod conformance;

pub use capabilities::{
    Degradation, FeaturePolicy, FeatureSupport, InteractionRequirements, Readiness, ResizeDelivery,
    TerminalCapabilities, TerminalConfig, TerminalFacts, TerminalRealization,
};
pub use driving::{Deadline, InteractionFeatures, ReadOutcome, WaitInterest, Wake};
