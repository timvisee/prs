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
    cmd_matches: &'a ArgMatches,
}

impl<'a> Remove<'a> {
    /// Construct a new remove action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the remove action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_recipients = RecipientsMatcher::with(self.cmd_matches).unwrap();
        let matcher_remove = RemoveMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_recipients.store()).map_err(Err::Store)?;
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        let tomb = store.tomb(!matcher_main.verbose(), matcher_main.verbose());
        let sync = store.sync();

        // Prepare tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb.prepare().map_err(Err::Tomb)?;

        // Prepare sync
        sync::ensure_ready(&sync, matcher_remove.allow_dirty());
        if !matcher_remove.no_sync() {
            sync.prepare()?;
        }

        let mut recipients = store.recipients().map_err(Err::Load)?;

        // Select key to remove
        let key = select::select_key(recipients.keys())
            .ok_or(Err::NoneSelected)?
            .clone();

        // Do not allow removing last recipient unless forcing
        if recipients.keys().len() == 1 && !matcher_main.force() {
            error::print_error_msg(
                "cannot remove last recipient from store, you will permanently loose access to it",
            );
            error::ErrorHintsBuilder::default()
                .force(true)
                .verbose(false)
                .build()
                .unwrap()
                .print();
            error::quit();
        }

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

        // Finalize sync
        if !matcher_remove.no_sync() {
            sync.finalize(format!("Remove recipient {}", key.fingerprint(true)))?;
        }

        // Finalize tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb.finalize().map_err(Err::Tomb)?;

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

    #[cfg(all(feature = "tomb", target_os = "linux"))]
    #[error("failed to prepare password store tomb for usage")]
    Tomb(#[source] anyhow::Error),

    #[error("no key selected")]
    NoneSelected,

    #[error("failed to load existing keys from store")]
    Load(#[source] anyhow::Error),

    #[error("failed to re-encrypt secrets in store")]
    Recrypt(#[source] anyhow::Error),
}
