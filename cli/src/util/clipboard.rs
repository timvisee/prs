use std::thread;
use std::time::Duration;

use anyhow::Result;
use copypasta_ext::prelude::*;
use notify_rust::{Hint, Notification};
use prs_lib::types::Plaintext;
use thiserror::Error;

use crate::util::error::{self, ErrorHintsBuilder};

/// Copy the given plain text to the user clipboard.
pub fn copy(data: &[u8]) -> Result<()> {
    let mut ctx = copypasta_ext::x11_fork::ClipboardContext::new().map_err(Err::Clipboard)?;
    ctx.set_contents(std::str::from_utf8(data).unwrap().into())
        .map_err(|err| Err::Clipboard(err).into())
}

/// Copy the given plain text to the user clipboard.
pub fn copy_timeout(data: &[u8], timeout: u64) -> Result<()> {
    if timeout == 0 {
        return copy(data);
    }

    copy_timeout_fallback(data, timeout)
}

/// Copy with timeout.
///
/// Simple fallback method using delay in console.
fn copy_timeout_fallback(data: &[u8], timeout: u64) -> Result<()> {
    // TODO: do not use x11 context here!
    use copypasta_ext::x11_fork::ClipboardContext;

    let mut ctx = ClipboardContext::new().map_err(Err::Clipboard)?;
    ctx.set_contents(std::str::from_utf8(data).unwrap().into())
        .map_err(Err::Clipboard)?;

    // TODO: clear clipboard on ctrl+c
    eprintln!("Waiting {} seconds to clear clipboard...", timeout);
    thread::sleep(Duration::from_secs(timeout));

    ctx.set_contents("".into()).map_err(Err::Clipboard)?;
    let _ = notify_cleared();

    Ok(())
}

/// Copy the given plain text to the user clipboard.
pub(crate) fn plaintext_copy(
    mut plaintext: Plaintext,
    first_line: bool,
    error_empty: bool,
    report_copied: bool,
    timeout: u64,
) -> Result<()> {
    if first_line {
        plaintext = plaintext.first_line()?;
    }

    // Do not copy empty secret
    if error_empty && plaintext.is_empty() {
        error::quit_error_msg(
            "secret is empty, did not copy to clipboard",
            ErrorHintsBuilder::default().force(true).build().unwrap(),
        )
    }

    copy_timeout(&plaintext.0, timeout).map_err(Err::CopySecret)?;

    // TODO: move into copy function, what to do?
    if report_copied {
        eprintln!("Secret copied to clipboard...");
    }

    Ok(())
}

/// Show notification to user about cleared clipboard.
fn notify_cleared() -> Result<()> {
    Notification::new()
        .appname(crate::APP_NAME)
        .summary(&format!("Clipboard cleared - {}", crate::APP_NAME))
        .body("Secret cleared from clipboard")
        .auto_icon()
        .icon("lock")
        .timeout(3000)
        .urgency(notify_rust::Urgency::Low)
        .hint(Hint::Category("presence.offline".into()))
        .show()?;
    Ok(())
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to set clipboard")]
    Clipboard(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("failed to copy secret to clipboard")]
    CopySecret(#[source] anyhow::Error),
}
