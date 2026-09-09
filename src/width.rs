//! Display cells are a deterministic policy, independently of text segmentation.
use crate::EditError;
use unicode_width::UnicodeWidthStr;

/// REPLAI's display-cell model. This is an assumption about terminal geometry,
/// not a claim of universal emulator/font agreement or active width probing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidthPolicy {
    /// `unicode-width` non-CJK policy: ambiguous East Asian characters are narrow;
    /// combining marks do not add cells, common emoji/ZWJ sequences use the
    /// dependency's sequence rules. Layout separately expands tabs and handles LF.
    UnicodeNarrow,
}
impl WidthPolicy {
    /// Measure safe single-line text in cells. Reject controls, tabs and newlines
    /// because their geometry requires layout/context, not a string-width sum.
    /// Extended-grapheme editing remains a separate segmentation operation.
    pub fn measure(self, text: &str) -> Result<usize, EditError> {
        if text.chars().any(char::is_control) {
            return Err(EditError::InvalidText);
        }
        Ok(cells(text))
    }
}
pub(crate) fn cells(text: &str) -> usize {
    text.width()
}
