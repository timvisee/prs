use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::store::Store;

use crate::cmd::matcher::{list::ListMatcher, Matcher};

/// List secrets action.
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
        let matcher_list = ListMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(crate::STORE_DEFAULT_ROOT).map_err(Err::Store)?;

        let mut secrets = store.secrets(matcher_list.query());
        secrets.sort_unstable_by(|a, b| a.name.cmp(&b.name));

        secrets.iter().for_each(|s| println!("{}", s.name));

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),
}
