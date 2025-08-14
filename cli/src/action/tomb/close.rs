use anyhow::Result;
use clap::ArgMatches;
use prs_lib::Store;
use thiserror::Error;

use crate::{
    cmd::matcher::{
        MainMatcher, Matcher,
        tomb::{TombMatcher, close::CloseMatcher},
    },
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
        let _matcher_tomb = TombMatcher::with(self.cmd_matches).unwrap();
        let matcher_close = CloseMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_main.store()).map_err(Err::Store)?;
        let tomb = store.tomb(
            !matcher_main.verbose(),
            matcher_main.verbose(),
            matcher_main.force(),
        );

        // Must be a tomb
        if !tomb.is_tomb() && !matcher_main.force() {
            if matcher_close.do_try() {
                return Ok(());
            }

            // TODO: error hint to initialize tomb
            error::quit_error_msg(
                "password store is not a tomb",
                ErrorHintsBuilder::from_matcher(&matcher_main)
                    .force(true)
                    .build()
                    .unwrap(),
            );
        }

        // Must be open
        if !tomb.is_open().map_err(Err::Close)? && !matcher_main.force() {
            if matcher_close.do_try() {
                return Ok(());
            }

            error::quit_error_msg(
                "password store tomb is not open",
                ErrorHintsBuilder::from_matcher(&matcher_main)
                    .force(true)
                    .build()
                    .unwrap(),
            );
        }

        if matcher_main.verbose() {
            eprintln!("Closing Tomb...");
        }

        // Close the tomb
        tomb.close().map_err(Err::Close)?;

        // Close any running close timers
        if let Err(err) = tomb.stop_timer() {
            error::print_error(err.context("failed to stop auto closing systemd timer, ignoring"));
        }

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
