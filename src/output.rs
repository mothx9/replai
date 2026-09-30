//! Scoped terminal output mechanics, without editing or application lifecycle.
use crate::{
    Block, Document, Error, Text, Theme, protocol::encode, render::Mutation, substrate::Transport,
    system::Resource,
};
use std::os::fd::AsFd;

/// One exclusive quiet-output lifetime between editor lifetimes. Descriptors
/// are duplicated; the same process lease and restoration owner as Interaction
/// apply. Signals retain their original terminal meaning. No thread is created.
pub struct OutputSession {
    resource: Resource,
    theme: Theme,
    feedback_rows: usize,
}
impl OutputSession {
    /// Acquire matching TTYs and suppress input echo, preserving ISIG/canonical mode.
    pub fn open(input: &impl AsFd, output: &impl AsFd) -> Result<Self, Error> {
        let (resource, theme) = Resource::acquire_quiet(input, output)?;
        Ok(Self {
            resource,
            theme,
            feedback_rows: 0,
        })
    }
    fn erase(&self) -> Vec<Mutation> {
        let mut out = Vec::new();
        if self.feedback_rows != 0 {
            if self.feedback_rows > 1 {
                out.push(Mutation::Up(self.feedback_rows - 1));
            }
            out.push(Mutation::CarriageReturn);
            for row in 0..self.feedback_rows {
                out.push(Mutation::ClearLine);
                if row + 1 < self.feedback_rows {
                    out.push(Mutation::Down(1));
                }
            }
            if self.feedback_rows > 1 {
                out.push(Mutation::Up(self.feedback_rows - 1));
            }
            out.push(Mutation::CarriageReturn);
        }
        out
    }
    /// Replace temporary feedback at a host-established line boundary. An empty
    /// text clears it. Hosts must clear before writing other output; arbitrary
    /// external writers are not coordinated by this deliberately single-producer scope.
    /// Validate/layout before terminal I/O. At most 16 KiB of semantic text.
    pub fn feedback(&mut self, text: Text) -> Result<(), Error> {
        let width = self.resource.dimensions()?.0;
        let empty = text.bytes() == 0;
        let d = Document::new(vec![Block::Paragraph(text)])?;
        let mut content = if empty {
            Vec::new()
        } else {
            d.mutations(width)?
        };
        let rows = if empty { 0 } else { d.layout(width)?.len() };
        // An empty paragraph has no rows. Keep the cursor on the last owned row,
        // so clearing feedback does not leave progress repaints in scrollback.
        if matches!(content.last(), Some(Mutation::Newline)) {
            content.pop();
        }
        let mut mutations = self.erase();
        mutations.extend(content);
        if let Err(e) = self
            .resource
            .write(encode(&mutations, self.theme).as_bytes())
        {
            let _ = self.close(false);
            return Err(e.into());
        }
        self.feedback_rows = rows;
        Ok(())
    }
    /// Clear temporary output, optionally discard queued input under explicit
    /// host policy, then restore captured modes. Cleanup attempts restoration
    /// even if clearing/discard fails. Drop retries restoration only.
    pub fn close(&mut self, discard_input: bool) -> Result<(), Error> {
        let clear = self
            .resource
            .cleanup_write(encode(&self.erase(), self.theme).as_bytes());
        self.feedback_rows = 0;
        let discard = if discard_input {
            self.resource.discard_input()
        } else {
            Ok(())
        };
        let restore = self.resource.restore();
        clear.and(discard).and(restore).map_err(Into::into)
    }
}
impl Drop for OutputSession {
    fn drop(&mut self) {
        let _ = self.close(false);
    }
}
