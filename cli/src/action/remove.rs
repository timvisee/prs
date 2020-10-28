use std::fs;

use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::store::Store;

use crate::cmd::matcher::{remove::RemoveMatcher, MainMatcher, Matcher};
use crate::util::{cli, error, skim, sync};

/// Remove secret action.
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
        let matcher_remove = RemoveMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_remove.store()).map_err(Err::Store)?;
        let sync = store.sync();

        sync::ensure_ready(&sync);
        sync.prepare()?;

        let secret =
            skim::select_secret(&store, matcher_remove.query()).ok_or(Err::NoneSelected)?;

        // Confirm removal
        if !matcher_main.force() {
            if !cli::prompt_yes(
                &format!("Remove '{}'?", secret.path.display()),
                Some(true),
                &matcher_main,
            ) {
                if matcher_main.verbose() {
                    eprintln!("Removal cancelled");
                }
                error::quit();
            }
        }

        // Remove secret
        fs::remove_file(&secret.path)
            .map(|_| ())
            .map_err(|err| Err::Remove(err))?;

        sync.finalize(format!("Remove secret from {}", secret.name))?;

        if !matcher_main.quiet() {
            eprintln!("Secret removed");
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
