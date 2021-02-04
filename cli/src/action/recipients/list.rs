use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::Store;

use crate::cmd::matcher::{recipients::RecipientsMatcher, MainMatcher, Matcher};

/// A recipients list action.
pub struct List<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> List<'a> {
    /// Construct a new list action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the list action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_recipients = RecipientsMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_recipients.store()).map_err(Err::Store)?;
        let recipients = store.recipients().map_err(Err::List)?;

        recipients
            .keys()
            .iter()
            .map(|key| {
                if !matcher_main.quiet() {
                    key.to_string()
                } else {
                    key.fingerprint(false)
                }
            })
            .for_each(|key| println!("{}", key,));

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to list store recipients")]
    List(#[source] anyhow::Error),
}
