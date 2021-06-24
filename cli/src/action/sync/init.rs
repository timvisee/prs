use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::Store;

use crate::{
    cmd::matcher::{sync::SyncMatcher, MainMatcher, Matcher},
    util::error::{self, ErrorHintsBuilder},
};

/// A sync init action.
pub struct Init<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Init<'a> {
    /// Construct a new init action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the init action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_sync = SyncMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_sync.store()).map_err(Err::Store)?;
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        let tomb = store.tomb(!matcher_main.verbose(), matcher_main.verbose());
        let sync = store.sync();

        // Prepare tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb.prepare().map_err(Err::Tomb)?;

        if sync.is_init() {
            error::quit_error_msg(
                "sync is already initialized",
                ErrorHintsBuilder::default().sync(true).build().unwrap(),
            );
        }

        // Initialize sync
        sync.init().map_err(Err::Init)?;

        // Run housekeeping
        crate::action::housekeeping::run::housekeeping(&store, true, false)
            .map_err(Err::Housekeeping)?;

        // Finalize tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb.finalize().map_err(Err::Tomb)?;

        if !matcher_main.quiet() {
            eprintln!("Sync initialized");
            if !sync.has_remote()? {
                eprintln!("Sync remote not configured, to set use: prs sync remote <GIT_URL>");
            }
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

    #[error("failed to initialize git sync")]
    Init(#[source] anyhow::Error),

    #[error("failed to run housekeeping tasks")]
    Housekeeping(#[source] anyhow::Error),
}
