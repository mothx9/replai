//! Isolated qualification crate; production sources compiled without a new product API.
#![allow(dead_code)]
#[path = "../../src/validation.rs"]
mod validation;
pub use validation::{
    Diagnostic, MAX_DIAGNOSTIC_BYTES, MAX_DIAGNOSTICS, MAX_VALIDATION_BYTES, SubmissionPolicy,
    ValidationDisposition, ValidationError, ValidationOutcome, ValidationResult,
};
#[path = "../../src/completion.rs"]
mod completion;
pub use completion::{
    CompletionAction, CompletionCandidate, CompletionError, CompletionSelection, CompletionSet,
    MAX_COMPLETION_BYTES, MAX_COMPLETION_CANDIDATES, MAX_COMPLETION_FIELD_BYTES,
};
#[path = "../../src/analysis_presentation.rs"]
mod analysis_presentation;
pub use analysis_presentation::{
    AnalysisPresentation, AnalysisPresentationError, AnalysisSpan, Hint, MAX_ANALYSIS_SPANS,
    MAX_HINT_BYTES,
};
#[path = "../../src/analysis.rs"]
mod analysis;
pub use analysis::{AnalysisOutcome, AnalysisSnapshot, DraftRevision};
#[path = "../../src/capabilities.rs"]
mod capabilities;
#[path = "../../src/width.rs"]
mod width;
pub use width::WidthPolicy;
#[path = "../../src/document.rs"]
mod document;
#[path = "../../src/driving.rs"]
mod driving;
pub use document::{Alignment, Block, Column, Document, ListItem, Severity, Span, Text};
#[path = "../../src/core.rs"]
mod core;
#[path = "../../src/event.rs"]
mod event;
#[path = "../../src/interaction.rs"]
mod interaction;
// Other system façades are intentionally absent. These internal components
// compile everywhere and execute through deterministic tests on every CI OS.
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
#[path = "../../src/actions.rs"]
mod actions;
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
#[path = "../../src/engine.rs"]
mod engine;
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
#[path = "../../src/input.rs"]
mod input;
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
#[path = "../../src/keymap.rs"]
mod keymap;
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
#[path = "../../src/presentation.rs"]
mod presentation;
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
#[path = "../../src/protocol.rs"]
mod protocol;
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
#[path = "../../src/render.rs"]
mod render;
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
#[path = "../../src/substrate.rs"]
mod substrate;
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[path = "../../src/system.rs"]
mod system;
#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
#[path = "../../src/terminal.rs"]
mod terminal;
pub use core::{EditError, Editor};
pub use event::{Error, Event};
pub use interaction::Interaction;
pub use presentation::{Foreground, Prompt, Role, Style, Theme};

pub use capabilities::{
    Degradation, FeaturePolicy, FeatureSupport, InteractionRequirements, Readiness, ResizeDelivery,
    TerminalCapabilities, TerminalConfig, TerminalFacts, TerminalRealization,
};
pub use driving::{Deadline, InteractionFeatures, ReadOutcome, WaitInterest, Wake};

mod campaigns;
mod model;
pub use campaigns::{editor_case, geometry_case, protocol_case, results_case};
#[cfg(unix)]
mod cabi;
#[cfg(unix)]
pub use cabi::cabi_case;

mod faults;
pub use faults::fault_campaign;
