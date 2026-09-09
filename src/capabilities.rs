//! Minimal terminal facts and admission policy, not universal capability negotiation.
use crate::{Error, InteractionFeatures, Theme};

/// Evidence posture for one terminal feature; assumptions are explicit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeatureSupport {
    /// The feature is unavailable.
    Unavailable,
    /// No evidence or explicit host assumption permits this feature.
    Unknown,
    /// The host accepts a terminal-protocol assumption, not a discovery result.
    Assumed,
    /// The host supplies affirmative capability evidence.
    Supported,
}
impl FeatureSupport {
    fn admitted(self) -> bool {
        matches!(self, Self::Assumed | Self::Supported)
    }
}

/// Host policy for an optional/degradable terminal feature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeaturePolicy {
    /// Do not enable the feature even when available.
    Disabled,
    /// Enable it when admitted, otherwise degrade without it.
    Preferred,
    /// Reject acquisition when the feature cannot be admitted.
    Required,
}

/// Host-supplied facts independent of environment variables or OS handles.
/// REPLAI separately verifies interactive, matching system resources and dimensions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalFacts {
    /// Cursor movement needed for the editing surface.
    pub cursor: FeatureSupport,
    /// Erase operations needed to preserve the editing surface.
    pub erase: FeatureSupport,
    /// Bracketed-paste protocol mode.
    pub bracketed_paste: FeatureSupport,
    /// Semantic foreground/intensity styling.
    pub styling: FeatureSupport,
}
impl TerminalFacts {
    /// Explicitly assume the existing VT interaction protocol. This is not probing.
    pub fn assumed_vt() -> Self {
        Self {
            cursor: FeatureSupport::Assumed,
            erase: FeatureSupport::Assumed,
            bracketed_paste: FeatureSupport::Assumed,
            styling: FeatureSupport::Assumed,
        }
    }
}

/// Admission configuration for new blocking/driven embeddings.
///
/// Cursor and erase are required. Styling and paste have independent policies.
/// Resize delivery is host-owned under driving and periodically observed under
/// compatibility polling; choosing features does not choose a scheduler.
#[derive(Clone, Copy, Debug)]
pub struct TerminalConfig {
    /// Terminal facts or explicit assumptions provided by the host.
    pub facts: TerminalFacts,
    /// Optional semantic styling policy.
    pub styling: FeaturePolicy,
    /// Optional bracketed-paste mode policy.
    pub bracketed_paste: FeaturePolicy,
    /// Semantic palette; an explicitly plain theme stays plain.
    pub theme: Theme,
}
impl TerminalConfig {
    /// Conservative environment convenience: nonempty TERM other than `dumb`
    /// supplies an explicit VT assumption. Missing/empty/dumb TERM does not
    /// establish cursor/erase support and acquisition refuses before raw mode.
    /// NO_COLOR disables styling. No emulator probing or signal handler is used.
    pub fn from_environment() -> Self {
        let term = std::env::var("TERM").ok();
        let known = term
            .as_deref()
            .is_some_and(|s| !s.is_empty() && s != "dumb");
        let mut facts = TerminalFacts::assumed_vt();
        if !known {
            facts.cursor = FeatureSupport::Unknown;
            facts.erase = FeatureSupport::Unknown;
            facts.bracketed_paste = FeatureSupport::Unknown;
            facts.styling = FeatureSupport::Unknown;
        }
        Self {
            facts,
            styling: if std::env::var_os("NO_COLOR").is_some() {
                FeaturePolicy::Disabled
            } else {
                FeaturePolicy::Preferred
            },
            bracketed_paste: FeaturePolicy::Preferred,
            theme: Theme::from_environment(true),
        }
    }
    /// Resolve required/optional features without acquiring a terminal or doing I/O.
    /// Unknown/unavailable cursor or erase always fails; optional loss is explicit
    /// in the returned features. A host-supplied plain theme disables styling.
    pub fn resolve(self) -> Result<(Theme, InteractionFeatures), Error> {
        if !self.facts.cursor.admitted() || !self.facts.erase.admitted() {
            return Err(Error::CapabilityMismatch(
                "interactive cursor and erase support required",
            ));
        }
        fn feature(support: FeatureSupport, policy: FeaturePolicy) -> Result<bool, Error> {
            match policy {
                FeaturePolicy::Disabled => Ok(false),
                FeaturePolicy::Preferred => Ok(support.admitted()),
                FeaturePolicy::Required if support.admitted() => Ok(true),
                FeaturePolicy::Required => Err(Error::CapabilityMismatch(
                    "required terminal feature unavailable",
                )),
            }
        }
        if self.styling == FeaturePolicy::Required && !self.theme.color {
            return Err(Error::CapabilityMismatch(
                "required styling conflicts with a plain theme",
            ));
        }
        let styling = feature(self.facts.styling, self.styling)? && self.theme.color;
        let bracketed_paste = feature(self.facts.bracketed_paste, self.bracketed_paste)?;
        let mut theme = self.theme;
        theme.color = styling;
        Ok((
            theme,
            InteractionFeatures {
                styling,
                bracketed_paste,
            },
        ))
    }
}

pub(crate) fn environment_theme(output_is_tty: bool) -> Theme {
    Theme::new(
        output_is_tty,
        std::env::var_os("NO_COLOR").is_some(),
        std::env::var("TERM").ok().as_deref(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn admission_distinguishes_required_optional_and_explicit_plain() {
        let mut c = TerminalConfig {
            facts: TerminalFacts::assumed_vt(),
            styling: FeaturePolicy::Preferred,
            bracketed_paste: FeaturePolicy::Preferred,
            theme: Theme::new(true, false, None),
        };
        assert!(c.resolve().unwrap().1.bracketed_paste);
        c.facts.bracketed_paste = FeatureSupport::Unavailable;
        assert!(!c.resolve().unwrap().1.bracketed_paste);
        c.bracketed_paste = FeaturePolicy::Required;
        assert!(matches!(c.resolve(), Err(Error::CapabilityMismatch(_))));
        c.bracketed_paste = FeaturePolicy::Disabled;
        c.facts.cursor = FeatureSupport::Unknown;
        assert!(c.resolve().is_err());
        c.facts.cursor = FeatureSupport::Supported;
        c.theme = Theme::new(true, true, None);
        assert!(!c.resolve().unwrap().1.styling);
        c.styling = FeaturePolicy::Required;
        assert!(c.resolve().is_err());
    }
}
