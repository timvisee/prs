use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{crypto::prelude::*, Store};
use thiserror::Error;

use crate::cmd::matcher::{
    recipients::{remove::RemoveMatcher, RecipientsMatcher},
    MainMatcher, Matcher,
};
#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;
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
        let _matcher_recipients = RecipientsMatcher::with(self.cmd_matches).unwrap();
        let matcher_remove = RemoveMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_main.store()).map_err(Err::Store)?;
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        let mut tomb = store.tomb(
            !matcher_main.verbose(),
            matcher_main.verbose(),
            matcher_main.force(),
        );
        let sync = store.sync();

        // Prepare tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::prepare_tomb(&mut tomb, &matcher_main).map_err(Err::Tomb)?;

        // Prepare sync
        sync::ensure_ready(&sync, matcher_remove.allow_dirty());
        if !matcher_remove.no_sync() {
            sync.prepare()?;
        }

        let mut recipients = store.recipients().map_err(Err::Load)?;

        // Select key to remove
        let key = select::select_key(recipients.keys(), None)
            .ok_or(Err::NoneSelected)?
            .clone();

        // Do not allow removing last recipient unless forcing
        if recipients.keys().len() == 1 && !matcher_main.force() {
            error::print_error_msg(
                "cannot remove last recipient from store, you will permanently loose access to it",
            );
            error::ErrorHintsBuilder::from_matcher(&matcher_main)
                .force(true)
                .verbose(false)
                .build()
                .unwrap()
                .print(false);
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
            crate::action::housekeeping::recrypt::recrypt_all(&store, &matcher_main)
                .map_err(Err::Recrypt)?;
        }

        // Finalize sync
        if !matcher_remove.no_sync() {
            sync.finalize(format!("Remove recipient {}", key.fingerprint(true)))?;
        }

        // Finalize tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::finalize_tomb(&mut tomb, &matcher_main, true).map_err(Err::Tomb)?;

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
