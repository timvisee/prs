use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::{
    store::Store,
    sync::{Readyness, Sync},
};

use crate::cmd::matcher::{
    sync::{init::InitMatcher, SyncMatcher},
    MainMatcher, Matcher,
};

/// A sync init action.
pub struct Init<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Init<'a> {
    /// Construct a new init action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the init action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_sync = SyncMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_sync.store()).map_err(Err::Store)?;
        let sync = store.sync();

        // Don't sync if not initialized or no remote, show help on how to set up
        match sync.readyness()? {
            Readyness::NoSync => {}
            _ => {
                if !matcher_main.quiet() {
                    println!("Sync already initialized, to sync use: prs sync");
                    no_remote_message(&sync)?;
                }
                crate::util::error::quit();
            }
        }

        // TODO: add default files (.gitattributes, etc)

        // Initialize git
        sync.init().map_err(Err::Init)?;

        if !matcher_main.quiet() {
            eprintln!("Sync initialized");
            no_remote_message(&sync)?;
        }

        Ok(())
    }
}

/// Show a no remote configured notice with instructions.
fn no_remote_message(sync: &Sync) -> Result<()> {
    if !sync.has_remote()? {
        eprintln!("Sync remote not configured, to set use: prs sync set-remote <GIT_URL>");
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to initialize git sync")]
    Init(#[source] anyhow::Error),
}
