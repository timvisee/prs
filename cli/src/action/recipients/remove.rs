use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::{crypto::prelude::*, Store};

use crate::cmd::matcher::{
    recipients::{remove::RemoveMatcher, RecipientsMatcher},
    MainMatcher, Matcher,
};
use crate::util::{cli, error, select, sync};

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
        let matcher_remove = RemoveMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_recipients.store()).map_err(Err::Store)?;
        let sync = store.sync();

        sync::ensure_ready(&sync);
        sync.prepare()?;

        let mut recipients = store.recipients().map_err(Err::Load)?;

        // Select key to remove
        let key = select::select_key(recipients.keys())
            .ok_or(Err::NoneSelected)?
            .clone();

        // Confirm removal
        if !matcher_main.force() {
            eprintln!("{}", key);
            if !cli::prompt_yes(
                &format!("Remove '{}'?", key.fingerprint(true),),
                Some(true),
                &matcher_main,
            ) {
                if matcher_main.verbose() {
                    eprintln!("Removal cancelled");
                }
                error::quit();
            }
        }

        recipients.remove(&key);
        recipients.save(&store)?;

        // Recrypt secrets
        if matcher_remove.recrypt() {
            crate::action::housekeeping::recrypt::recrypt_all(
                &store,
                matcher_main.quiet(),
                matcher_main.verbose(),
            )
            .map_err(Err::Recrypt)?;
        }

        sync.finalize(format!("Remove recipient {}", key.fingerprint(true)))?;

        if !matcher_main.quiet() {
            eprintln!("Removed recipient: {}", key);
        }

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

    #[error("failed to re-encrypt secrets in store")]
    Recrypt(#[source] anyhow::Error),
}
