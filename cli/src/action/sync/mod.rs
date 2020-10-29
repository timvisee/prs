pub mod init;
pub mod remote;

use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::{
    store::Store,
    sync::{Readyness, Sync as StoreSync},
    Recipients,
};

use crate::{
    cmd::matcher::{sync::SyncMatcher, MainMatcher, Matcher},
    util::sync,
};

/// Sync secrets action.
pub struct Sync<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Sync<'a> {
    /// Construct a new sync action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the sync action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_sync = SyncMatcher::with(self.cmd_matches).unwrap();

        if matcher_sync.cmd_init().is_some() {
            return init::Init::new(self.cmd_matches).invoke();
        }

        if matcher_sync.cmd_remote().is_some() {
            return remote::Remote::new(self.cmd_matches).invoke();
        }

        let store = Store::open(matcher_sync.store()).map_err(Err::Store)?;
        let sync = StoreSync::new(&store);

        // Don't sync if not initialized or no remote, show help on how to set up
        match sync.readyness()? {
            Readyness::NoSync => {
                if !matcher_main.quiet() {
                    println!("Sync not configured, to initialize use: prs sync init");
                }
                crate::util::error::quit();
            }
            _ if !sync.has_remote()? => {
                // TODO: this should be a warning instead, should continue
                if !matcher_main.quiet() {
                    println!("Sync remote not configured, to set use: prs sync remote <GIT_URL>");
                }
                crate::util::error::quit();
            }
            _ => {}
        }

        sync::ensure_ready(&sync);

        // Prepare, commit, finalize
        sync.prepare()?;
        sync.finalize("Sync dirty changes")?;

        // TODO: assume changed for now, fetch this state from syncer
        let changed = true;

        // Were done if nothing was changed
        if !changed {
            if !matcher_main.quiet() {
                eprintln!("Already up to date");
            }
            return Ok(());
        }

        // Import new keys
        Recipients::import_missing_keys_from_store(&store).map_err(Err::ImportRecipients)?;

        // TODO: assert not-dirty state?

        if !matcher_main.quiet() {
            eprintln!("Sync complete");
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to import store recipients")]
    ImportRecipients(#[source] anyhow::Error),
}
