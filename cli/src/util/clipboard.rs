use anyhow::Result;
use copypasta_ext::{prelude::*, x11_fork::ClipboardContext};
use thiserror::Error;

/// Copy the given plain text to the user clipboard.
// TODO: move to clipboard module
// TODO: create function to copy with timeout
pub fn copy(data: &[u8]) -> Result<()> {
    let mut ctx = ClipboardContext::new().map_err(Err::Clipboard)?;
    ctx.set_contents(std::str::from_utf8(data).unwrap().into())
        .map_err(|err| Err::Clipboard(err).into())
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to copy secret to clipboard")]
    Clipboard(#[source] Box<dyn std::error::Error + Send + Sync>),
}
