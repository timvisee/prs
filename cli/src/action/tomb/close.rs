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
        let tomb = store.tomb();

        // Must be a tomb
        if !tomb.is_tomb() && !matcher_main.force() {
            // TODO: error hint to initialize tomb
            error::quit_error_msg(
                "password store is not a tomb",
                ErrorHintsBuilder::default().force(true).build().unwrap(),
            );
        }

        // Must be open
        if !tomb.is_open().map_err(Err::Close)? && !matcher_main.force() {
            error::quit_error_msg(
                "password store tomb is not open",
                ErrorHintsBuilder::default().force(true).build().unwrap(),
            );
        }

        if matcher_main.verbose() {
            eprintln!("Closing Tomb...");
        }

        // Close the tomb
        tomb.close().map_err(Err::Close)?;

        if !matcher_main.quiet() {
            eprintln!("Password store Tomb closed");
        }

        // TODO: show warning if there are still files in tomb directory

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to close password store tomb")]
    Close(#[source] anyhow::Error),
}
