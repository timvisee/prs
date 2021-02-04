use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{
    crypto::{self, prelude::*},
    store::Store,
};
use thiserror::Error;

use crate::cmd::matcher::{copy::CopyMatcher, MainMatcher, Matcher};
use crate::util::{clipboard, skim};

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

        let store = Store::open(matcher_copy.store()).map_err(Err::Store)?;
        let secret = skim::select_secret(&store, matcher_copy.query()).ok_or(Err::NoneSelected)?;

        let mut plaintext = crypto::context(crypto::PROTO)?
            .decrypt_file(&secret.path)
            .map_err(Err::Read)?;

        // Trim plaintext to property or first line
        if let Some(property) = matcher_copy.property() {
            plaintext = plaintext.property(property).map_err(Err::Property)?;
        } else if !matcher_copy.all() {
            plaintext = plaintext.first_line()?;
        }

        clipboard::plaintext_copy(
            plaintext,
            false,
            !matcher_main.force(),
            !matcher_main.quiet(),
            matcher_copy.timeout()?,
        )
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("no secret selected")]
    NoneSelected,

    #[error("failed to read secret")]
    Read(#[source] anyhow::Error),

    #[error("failed to select property from secret")]
    Property(#[source] anyhow::Error),
}
