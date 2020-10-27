use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::store::Store;

use crate::cmd::matcher::{recipients::RecipientsMatcher, MainMatcher, Matcher};
use crate::util;

/// A recipients remove action.
pub struct Remove<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Remove<'a> {
    /// Construct a new remove action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the remove action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_recipients = RecipientsMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_recipients.store()).map_err(Err::Store)?;
        let mut recipients = store.recipients().map_err(Err::Load)?;

        // Select key to remove
        let key = util::skim_select_key(recipients.keys())
            .ok_or(Err::NoneSelected)?
            .clone();
        recipients.remove(&key);

        recipients.save(&store)?;

        if !matcher_main.quiet() {
            eprintln!("Removed recipient: {}", key);
        }

        // TODO: recrypt everything for new recipient

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("no key selected")]
    NoneSelected,

    #[error("failed to load existing keys from store")]
    Load(#[source] anyhow::Error),
}
