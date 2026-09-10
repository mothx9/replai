//! Draft provenance shared by all host analyses, independent of terminals.
use std::sync::Arc;

/// Identity of one analysis-visible state within its originating retained Editor.
///
/// Equality is meaningful only within that editor's lifetime. Route results back
/// to their originating editor; unrelated editors do not share an identity domain.
/// No ordering, arithmetic, clock or global uniqueness is promised.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DraftRevision(u128);
impl DraftRevision {
    pub(crate) const INITIAL: Self = Self(0);
    #[cfg(test)]
    pub(crate) const EXHAUSTED: Self = Self(u128::MAX);
    pub(crate) fn advance(&mut self) {
        self.0 = self.0.checked_add(1).expect("draft revision exhausted");
    }
}

/// An immutable, coherent draft view which may outlive subsequent live editing.
///
/// Creation copies the draft once. Cloning shares its immutable text allocation;
/// the host can parse once and distribute derived results under the same revision.
/// This value is Send + Sync; it does not make the live interaction shared.
#[derive(Clone, Debug)]
pub struct AnalysisSnapshot {
    revision: DraftRevision,
    text: Arc<str>,
    cursor: usize,
}
impl AnalysisSnapshot {
    pub(crate) fn new(revision: DraftRevision, text: &str, cursor: usize) -> Self {
        Self {
            revision,
            text: Arc::from(text),
            cursor,
        }
    }
    /// Originating draft identity, scoped to its retained editor.
    pub fn revision(&self) -> DraftRevision {
        self.revision
    }
    /// Exact unmodified draft text, without host semantic context.
    pub fn text(&self) -> &str {
        &self.text
    }
    /// UTF-8 byte offset at an extended-grapheme boundary in this snapshot.
    pub fn cursor(&self) -> usize {
        self.cursor
    }
}

/// Revision-bound replacement outcome; malformed current edits remain edit errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisOutcome {
    /// The originating revision was current and the replacement was admitted.
    /// A replacement that changes neither text nor cursor retains its revision.
    Applied,
    /// The originating revision is no longer current. Nothing was changed.
    Stale,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exhausted_revision_never_wraps() {
        let mut r = DraftRevision(u128::MAX);
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| r.advance())).is_err());
        assert_eq!(r, DraftRevision(u128::MAX));
    }
}
