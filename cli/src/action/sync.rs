use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::{
    store::Store,
    sync::{Readyness, Sync as StoreSync},
};

use crate::cmd::matcher::{sync::SyncMatcher, MainMatcher, Matcher};

use crate::util::{self, ErrorHintsBuilder};

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

        ensure_ready(&sync);

        // Prepare, commit, finalize
        sync.prepare()?;
        sync.finalize(format!("Sync dirty changes with {}", crate_name!()))?;

        // TODO: assert not-dirty state?

        if !matcher_main.quiet() {
            eprintln!("Sync complete");
        }

        Ok(())
    }
}

/// Ensure the store is ready, otherwise quit.
fn ensure_ready(sync: &StoreSync) {
    let readyness = match sync.readyness() {
        Ok(readyness) => readyness,
        Err(err) => {
            util::quit_error(
                err.context("failed to query store sync readyness state"),
                ErrorHintsBuilder::default().git(true).build().unwrap(),
            );
        }
    };

    let msg = match readyness {
        Readyness::Ready | Readyness::NoSync => return,
        Readyness::Dirty => "store git repository has uncommitted changes".into(),
        Readyness::GitState(state) => {
            format!("store git repository is in unfinished state: {:?}", state)
        }
    };

    util::quit_error_msg(msg, ErrorHintsBuilder::default().git(true).build().unwrap());
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),
}
