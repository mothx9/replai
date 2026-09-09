//! Acquisition-time facts, protocol assumptions, requirements and degradation.
use crate::{Error, InteractionFeatures, Theme, WidthPolicy};

/// Evidence posture for a terminal property; assumptions never become discovery.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeatureSupport {
    /// Affirmatively unavailable.
    Unavailable,
    /// No affirmative evidence or accepted assumption.
    Unknown,
    /// An explicitly accepted protocol or host assumption.
    Assumed,
    /// Affirmative evidence supplied by a backend or host.
    Supported,
}
impl FeatureSupport {
    fn admitted(self) -> bool {
        matches!(self, Self::Assumed | Self::Supported)
    }
}

/// Policy for independently optional terminal features.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeaturePolicy {
    /// Do not use the feature, regardless of support.
    Disabled,
    /// Use the feature if admitted; otherwise report degradation.
    Preferred,
    /// Refuse when the feature cannot be used.
    Required,
}

/// Protocol properties, independent of resource observations and user policy.
/// `Supported` is a host assertion; REPLAI does not actively probe VT support.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalFacts {
    /// Relative/absolute cursor positioning, CR/LF and conventional cell advance.
    /// The current VT renderer also assumes conventional autowrap/scroll behavior.
    pub cursor: FeatureSupport,
    /// Line/range erasure and clear-to-home needed for redraw and Ctrl-L.
    pub erase: FeatureSupport,
    /// Framed paste mode; independent of ordinary text input.
    pub bracketed_paste: FeatureSupport,
    /// The current semantic palette: SGR intensity and 256-color foregrounds.
    pub styling: FeatureSupport,
}
impl TerminalFacts {
    /// Explicitly assume the current VT protocol. No property is discovered.
    pub fn assumed_vt() -> Self {
        Self {
            cursor: FeatureSupport::Assumed,
            erase: FeatureSupport::Assumed,
            bracketed_paste: FeatureSupport::Assumed,
            styling: FeatureSupport::Assumed,
        }
    }
    /// Interpret TERM only as a hint. A nonempty value other than `dumb` permits
    /// an assumption, never `Supported`; absence/dumb leaves protocol facts unknown.
    pub fn from_term_hint(term: Option<&str>) -> Self {
        if term.is_some_and(|t| !t.is_empty() && t != "dumb") {
            Self::assumed_vt()
        } else {
            Self {
                cursor: FeatureSupport::Unknown,
                erase: FeatureSupport::Unknown,
                bracketed_paste: FeatureSupport::Unknown,
                styling: FeatureSupport::Unknown,
            }
        }
    }
}

/// Readiness realization, not current readability or an OS handle type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Readiness {
    /// No usable input wait mechanism is established.
    Unavailable,
    /// The backend can wait internally, but exposes no host waitable source.
    BackendManaged,
    /// A borrowed source can be registered with a host wait mechanism.
    Waitable,
}

/// Resource observations supplied by a backend, or explicit assertions in a
/// controlled/virtual host. Native acquisition always supplies its own values;
/// protocol configuration cannot override negative native observations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalRealization {
    /// Interactive input is available.
    pub input: FeatureSupport,
    /// Interactive output is available. Unavailable output forces plain presentation.
    pub output: FeatureSupport,
    /// Input mode can be captured, configured and restored by the resource owner.
    pub restoration: FeatureSupport,
    /// Current columns/rows, when known. Editing requires at least 2 of each.
    pub dimensions: Option<(usize, usize)>,
    /// Dimensions can be queried again; not a promise of resize notifications.
    pub dimension_query: FeatureSupport,
    /// Backend wait support; independent of decoder deadlines.
    pub readiness: Readiness,
}
impl TerminalRealization {
    /// A captured/noninteractive stream. A document supplies its own layout width.
    pub fn captured() -> Self {
        Self {
            input: FeatureSupport::Unavailable,
            output: FeatureSupport::Unavailable,
            restoration: FeatureSupport::Unavailable,
            dimensions: None,
            dimension_query: FeatureSupport::Unavailable,
            readiness: Readiness::Unavailable,
        }
    }
    /// Affirm properties of a controlled interactive realization. Native callers
    /// obtain these from acquisition instead; this constructor performs no probing.
    pub fn interactive(dimensions: (usize, usize)) -> Self {
        Self {
            input: FeatureSupport::Supported,
            output: FeatureSupport::Supported,
            restoration: FeatureSupport::Supported,
            dimensions: Some(dimensions),
            dimension_query: FeatureSupport::Supported,
            readiness: Readiness::Waitable,
        }
    }
}

/// Required mechanics, independently of optional feature policy and host meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionRequirements {
    /// Output only: no input, cursor, erase, raw mode, wait or resize requirement.
    Presentation,
    /// Editable input: paired interactive resources, restoration, dimensions,
    /// cursor/erase protocol and at least backend-managed readiness.
    Editing,
    /// Editing plus a waitable source suitable for an external host reactor.
    Driven,
}

/// Available resize delivery paths, independently of the current dimensions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizeDelivery {
    /// No automatic geometry refresh is established (captured or fixed surface).
    Fixed,
    /// `poll` queries periodically; driven hosts notify via `Wake::Resize`.
    /// No signal handler or periodic driven timer is installed by this capability.
    PollOrHostNotification,
}

/// Why an optional feature is not active. This is not a loss of editor contents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Degradation {
    /// The feature was admitted without loss.
    None,
    /// Host/user policy (including an explicitly plain theme) disabled it.
    Disabled,
    /// Available facts did not establish the preferred feature.
    Unavailable,
}

/// Inspectable acquisition snapshot. Facts retain their evidence posture; enabled
/// features describe policy results. No handles, private mutations or callbacks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalCapabilities {
    /// Protocol facts/assumptions, unchanged by NO_COLOR or other feature policy.
    pub facts: TerminalFacts,
    /// Resource observations; native callers cannot override these through config.
    pub realization: TerminalRealization,
    /// Requirements admitted at open (not a scheduler owned by REPLAI).
    pub requirements: InteractionRequirements,
    /// Effective feature switches used by prompt, structured output and modes.
    pub features: InteractionFeatures,
    /// Styling loss or explicit disablement.
    pub styling_degradation: Degradation,
    /// Paste loss or explicit disablement. Ordinary input remains editable, but
    /// unframed multiline paste has no atomicity guarantee and Enter can submit.
    pub paste_degradation: Degradation,
    /// Available geometry refresh paths. The host supplies driven notifications.
    pub resize: ResizeDelivery,
    /// Deterministic cell policy; it is not an emulator/font discovery result.
    pub width_policy: WidthPolicy,
}

/// Protocol facts and independent optional policies, shared by every tier.
///
/// Environment convenience captures hints/policy once. Explicit configuration is
/// authoritative for protocol assumptions and presentation policy; observed
/// native resources are always authoritative for resource suitability.
#[derive(Clone, Copy, Debug)]
pub struct TerminalConfig {
    /// Protocol facts or explicit host assumptions.
    pub facts: TerminalFacts,
    /// Styling admission/degradation policy.
    pub styling: FeaturePolicy,
    /// Paste mode admission/degradation policy.
    pub bracketed_paste: FeaturePolicy,
    /// Role palette. An explicitly plain theme is a styling policy veto.
    pub theme: Theme,
}
impl TerminalConfig {
    /// Capture TERM as a hint and NO_COLOR presence (even empty) as policy.
    /// Explicit host facts can subsequently replace hints without changing policy.
    pub fn from_environment() -> Self {
        let term = std::env::var("TERM").ok();
        let no_color = std::env::var_os("NO_COLOR").is_some();
        Self {
            facts: TerminalFacts::from_term_hint(term.as_deref()),
            styling: if no_color {
                FeaturePolicy::Disabled
            } else {
                FeaturePolicy::Preferred
            },
            bracketed_paste: FeaturePolicy::Preferred,
            // Keep the palette independent of the TERM hint. Replacing unknown
            // protocol facts with stronger host evidence must be effective.
            theme: Theme::new(true, no_color, None),
        }
    }
    /// Explicit legacy VT assumption used by `open`/`open_with_theme` and C ABI 1.
    /// TERM=dumb may suppress styling through the supplied theme; it does not
    /// convert these retained compatibility assumptions into discovered support.
    pub fn compatibility(theme: Theme) -> Self {
        Self {
            facts: TerminalFacts::assumed_vt(),
            styling: FeaturePolicy::Preferred,
            bracketed_paste: FeaturePolicy::Preferred,
            theme,
        }
    }
    /// Resolve protocol policy only, preserving the original convenience API.
    /// This performs no resource observation and does not establish TTY suitability.
    pub fn resolve(self) -> Result<(Theme, InteractionFeatures), Error> {
        let (theme, features, _, _) = self.features(true, true)?;
        Ok((theme, features))
    }
    /// Pure, allocation-free admission for explicit resource evidence and requirements.
    /// No environment reads, OS calls or active terminal probes occur here.
    pub fn resolve_for(
        self,
        realization: TerminalRealization,
        requirements: InteractionRequirements,
    ) -> Result<(Theme, TerminalCapabilities), Error> {
        let editing = requirements != InteractionRequirements::Presentation;
        if editing {
            if !realization.input.admitted() || !realization.output.admitted() {
                return Err(Error::CapabilityMismatch(
                    "interactive input and output required",
                ));
            }
            if !realization.restoration.admitted() {
                return Err(Error::CapabilityMismatch("restorable input mode required"));
            }
            if !realization
                .dimensions
                .is_some_and(|(c, r)| c >= 2 && r >= 2)
            {
                return Err(Error::CapabilityMismatch(
                    "terminal dimensions of at least 2 columns and 2 rows required",
                ));
            }
            if realization.readiness == Readiness::Unavailable {
                return Err(Error::CapabilityMismatch("input readiness required"));
            }
            if requirements == InteractionRequirements::Driven
                && realization.readiness != Readiness::Waitable
            {
                return Err(Error::CapabilityMismatch(
                    "host waitable readiness source required",
                ));
            }
        }
        let (theme, features, styling_degradation, paste_degradation) =
            self.features(editing, realization.output.admitted())?;
        Ok((
            theme,
            TerminalCapabilities {
                facts: self.facts,
                realization,
                requirements,
                features,
                styling_degradation,
                paste_degradation,
                resize: if editing && realization.dimension_query.admitted() {
                    ResizeDelivery::PollOrHostNotification
                } else {
                    ResizeDelivery::Fixed
                },
                width_policy: WidthPolicy::UnicodeNarrow,
            },
        ))
    }
    fn features(
        self,
        editing: bool,
        output: bool,
    ) -> Result<(Theme, InteractionFeatures, Degradation, Degradation), Error> {
        if editing && (!self.facts.cursor.admitted() || !self.facts.erase.admitted()) {
            return Err(Error::CapabilityMismatch(
                "interactive cursor and erase support required",
            ));
        }
        let (styling, sd) = feature(
            self.facts.styling.admitted() && output,
            self.styling,
            !self.theme.color,
            "required styling unavailable or disabled",
        )?;
        let (bracketed_paste, pd) = if editing {
            feature(
                self.facts.bracketed_paste.admitted(),
                self.bracketed_paste,
                false,
                "required bracketed paste unavailable or disabled",
            )?
        } else {
            (false, Degradation::Disabled)
        };
        let mut theme = self.theme;
        theme.color = styling;
        Ok((
            theme,
            InteractionFeatures {
                styling,
                bracketed_paste,
            },
            sd,
            pd,
        ))
    }
}
fn feature(
    available: bool,
    policy: FeaturePolicy,
    veto: bool,
    error: &'static str,
) -> Result<(bool, Degradation), Error> {
    if policy == FeaturePolicy::Disabled {
        return Ok((false, Degradation::Disabled));
    }
    if policy == FeaturePolicy::Required && (!available || veto) {
        return Err(Error::CapabilityMismatch(error));
    }
    Ok(if veto {
        (false, Degradation::Disabled)
    } else if available {
        (true, Degradation::None)
    } else {
        (false, Degradation::Unavailable)
    })
}

// Compatibility constructor policy is centralized here too. `None` retains the
// historical caller-supplied styling assumption; this is not environment discovery.
pub(crate) fn styling_allowed(output: bool, no_color: bool, term: Option<&str>) -> bool {
    output && !no_color && term != Some("dumb")
}
pub(crate) fn environment_theme(output: bool) -> Theme {
    Theme::new(
        output,
        std::env::var_os("NO_COLOR").is_some(),
        std::env::var("TERM").ok().as_deref(),
    )
}
