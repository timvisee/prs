use clap::ArgMatches;
use copypasta_ext::prelude::*;
use copypasta_ext::x11_fork::ClipboardContext;

use anyhow::Result;
use prs_lib::{store::Store, types::Plaintext};
use thiserror::Error;

use crate::cmd::matcher::{copy::CopyMatcher, Matcher};

/// A file copy action.
pub struct Copy<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Copy<'a> {
    /// Construct a new copy action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the copy action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_copy = CopyMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(crate::STORE_DEFAULT_ROOT);

        // TODO: do not error on none selected
        let secrets = store.secrets(matcher_copy.query());
        let secret = crate::select_secret(&secrets).ok_or(Err::NoneSelected)?;

        // TODO: attach decrypt error here
        let mut plaintext =
            prs_lib::crypto::decrypt_file(&secret.path).map_err(|_| Err::Decrypt)?;

        // Trim plaintext to first line
        if !matcher_copy.all() {
            plaintext = plaintext.first_line().map_err(Err::FirstLine)?;
        }

        copy(plaintext)
    }
}

/// Copy the given plain text to the user clipboard.
// TODO: clear clipboard after timeout
fn copy(plaintext: Plaintext) -> Result<()> {
    let mut ctx = ClipboardContext::new().map_err(Err::Clipboard)?;
    ctx.set_contents(plaintext.to_str().unwrap().into())
        .map_err(Err::Clipboard)?;

    eprintln!("Secret copied to clipboard...");
    Ok(())
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to grab first line of secret")]
    FirstLine(#[source] std::str::Utf8Error),

    #[error("no secret selected")]
    NoneSelected,

    #[error("failed to decrypt secret")]
    Decrypt,

    #[error("failed to copy secret to clipboard")]
    Clipboard(#[source] Box<dyn std::error::Error + Send + Sync>),
}
