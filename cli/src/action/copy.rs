use clap::ArgMatches;
use copypasta_ext::prelude::*;
use copypasta_ext::x11_fork::ClipboardContext;

use anyhow::Result;
use prs_lib::{store::Store, types::Plaintext};
use thiserror::Error;

use crate::cmd::matcher::{copy::CopyMatcher, MainMatcher, Matcher};
use crate::util::{self, ErrorHintsBuilder};

/// Copy secret to clipboard action.
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
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_copy = CopyMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(crate::STORE_DEFAULT_ROOT).map_err(Err::Store)?;
        let secret = util::select_secret(&store, matcher_copy.query()).ok_or(Err::NoneSelected)?;

        let plaintext = prs_lib::crypto::decrypt_file(&secret.path).map_err(Err::Read)?;

        smart_copy(
            plaintext,
            !matcher_copy.all(),
            !matcher_main.force(),
            !matcher_main.quiet(),
        )
    }
}

/// Copy the given plain text to the user clipboard.
// TODO: move to shared module
// TODO: clear clipboard after timeout
pub(crate) fn smart_copy(
    mut plaintext: Plaintext,
    first_line: bool,
    error_empty: bool,
    report_copied: bool,
) -> Result<()> {
    if first_line {
        plaintext = plaintext.first_line()?;
    }

    // Do not copy empty secret
    if error_empty && plaintext.is_empty() {
        util::quit_error_msg(
            "Secret is empty, did not copy to clipboard",
            ErrorHintsBuilder::default().force(true).build().unwrap(),
        )
    }

    copy(plaintext)?;

    if report_copied {
        eprintln!("Secret copied to clipboard...");
    }

    Ok(())
}

/// Copy the given plain text to the user clipboard.
// TODO: move to shared module
// TODO: clear clipboard after timeout
fn copy(plaintext: Plaintext) -> Result<()> {
    let mut ctx = ClipboardContext::new().map_err(Err::Clipboard)?;
    ctx.set_contents(plaintext.to_str().unwrap().into())
        .map_err(|err| Err::Clipboard(err).into())
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("no secret selected")]
    NoneSelected,

    #[error("failed to read secret")]
    Read(#[source] anyhow::Error),

    #[error("failed to copy secret to clipboard")]
    Clipboard(#[source] Box<dyn std::error::Error + Send + Sync>),
}
