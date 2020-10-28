use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::{
    store::Store,
    sync::{Readyness, Sync as StoreSync},
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

        let store = Store::open(matcher_sync.store()).map_err(Err::Store)?;
        let sync = StoreSync::new(&store);

        // Don't sync if not initialized or no remote, show help on how to set up
        match sync.readyness()? {
            Readyness::NoSync => {
                println!("Sync not configured, to initialize use: prs git init");
                crate::util::error::quit();
            }
            _ if !sync.has_remote()? => {
                println!(
                    "Sync remote not configured, to set use: prs git remote add origin <GIT_URL>"
                );
                crate::util::error::quit();
            }
            _ => {}
        }

        sync::ensure_ready(&sync);

        // TODO: show error if sync is not initialized

        // Prepare, commit, finalize
        sync.prepare()?;
        sync.finalize("Sync dirty changes")?;

        // TODO: assert not-dirty state?

        // TODO: sync keys

        if !matcher_main.quiet() {
            eprintln!("Already up to date");
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),
}
