use anyhow::Result;
use clap::ArgMatches;
use prs_lib::Store;
use thiserror::Error;

use crate::{
    cmd::matcher::{
        tomb::{open::OpenMatcher, TombMatcher},
        MainMatcher, Matcher,
    },
    util::error::{self, ErrorHintsBuilder},
};

/// A tomb open action.
pub struct Open<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Open<'a> {
    /// Construct a new init action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the init action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_tomb = TombMatcher::with(self.cmd_matches).unwrap();
        let matcher_open = OpenMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_tomb.store()).map_err(Err::Store)?;
        let tomb = store.tomb();
        let timer = matcher_open.timer();

        // TODO: show warning if there already are files in tomb directory?

        // Must be a tomb
        if !tomb.is_tomb() && !matcher_main.force() {
            // TODO: error hint to initialize tomb
            error::quit_error_msg(
                "password store is not a tomb",
                ErrorHintsBuilder::default().force(true).build().unwrap(),
            );
        }

        // Must not be already open
        if tomb.is_open().map_err(Err::Open)? && !matcher_main.force() {
            error::quit_error_msg(
                "password store tomb is already open",
                ErrorHintsBuilder::default().force(true).build().unwrap(),
            );
        }

        if matcher_main.verbose() {
            eprintln!("Opening Tomb...");
        }

        // Open the tomb
        tomb.open().map_err(Err::Open)?;

        // Start timer
        if let Some(timer) = timer {
            if let Err(err) = tomb.stop_timer() {
                error::print_error(err.context(
                    "failed to stop existing timer to automatically close password store tomb, ignoring...",
                ));
            }
            tomb.start_timer(timer, true).map_err(Err::Timer)?;
        }

        if !matcher_main.quiet() {
            eprintln!("Password store Tomb opened");
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to open password store tomb")]
    Open(#[source] anyhow::Error),

    #[error("failed to start timer to automatically close password store tomb")]
    Timer(#[source] anyhow::Error),
}
