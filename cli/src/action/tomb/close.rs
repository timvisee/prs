use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::Store;

use crate::{
    cmd::matcher::{tomb::TombMatcher, MainMatcher, Matcher},
    util::error::{self, ErrorHintsBuilder},
};

/// A tomb close action.
pub struct Close<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Close<'a> {
    /// Construct a new init action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the init action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_tomb = TombMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_tomb.store()).map_err(Err::Store)?;
        let sync = store.sync();

        // TODO: ensure tomb is opened

        // if sync.is_init() {
        //     error::quit_error_msg(
        //         "sync is already initialized",
        //         ErrorHintsBuilder::default().sync(true).build().unwrap(),
        //     );
        // }

        // // Initialize git
        // sync.init().map_err(Err::Init)?;

        // if !matcher_main.quiet() {
        //     eprintln!("Sync initialized");
        //     if !sync.has_remote()? {
        //         eprintln!("Sync remote not configured, to set use: prs sync remote <GIT_URL>");
        //     }
        // }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to initialize git sync")]
    Init(#[source] anyhow::Error),
}
