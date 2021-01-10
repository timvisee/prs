use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::store::{Secret, SecretIterConfig, Store};

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

        let store = Store::open(matcher_list.store()).map_err(Err::Store)?;

        // List aliases based on filters, sort the list
        let config = SecretIterConfig {
            find_files: !matcher_list.only_aliases(),
            find_symlink_files: !matcher_list.only_non_aliases(),
        };
        let mut secrets: Vec<Secret> = store
            .secret_iter_config(config)
            .filter_name(matcher_list.query())
            .collect();
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
