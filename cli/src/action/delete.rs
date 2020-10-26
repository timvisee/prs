use std::fs;

use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::store::Store;

use crate::cmd::matcher::{delete::DeleteMatcher, MainMatcher, Matcher};
use crate::util;

/// Delete secret action.
pub struct Delete<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Delete<'a> {
    /// Construct a new delete action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the delete action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_delete = DeleteMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(crate::STORE_DEFAULT_ROOT).map_err(Err::Store)?;

        let secrets = store.secrets(matcher_delete.query());
        let secret = crate::select_secret(&secrets).ok_or(Err::NoneSelected)?;

        // Cofnirm deletion
        if !matcher_main.force() {
            if !util::prompt_yes(
                &format!(
                    "Are you sure you want to delete '{}'?",
                    secret.path.display()
                ),
                Some(true),
                &matcher_main,
            ) {
                if matcher_main.verbose() {
                    eprintln!("Deletion cancelled");
                }
                util::quit();
            }
        }

        // Delete secret
        fs::remove_file(&secret.path)
            .map(|_| ())
            .map_err(|err| Err::Remove(err))?;

        if !matcher_main.quiet() {
            eprintln!("Secret deleted");
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("no secret selected")]
    NoneSelected,

    #[error("failed to remove secret file")]
    Remove(#[source] std::io::Error),
}
