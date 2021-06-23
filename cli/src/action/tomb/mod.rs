pub mod close;
pub mod init;
pub mod open;
pub mod status;

use anyhow::Result;
use clap::ArgMatches;

use crate::cmd::matcher::{tomb::TombMatcher, Matcher};

/// Tomb management action.
pub struct Tomb<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Tomb<'a> {
    /// Construct a new sync action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the sync action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_tomb = TombMatcher::with(self.cmd_matches).unwrap();

        if matcher_tomb.cmd_init().is_some() {
            return init::Init::new(self.cmd_matches).invoke();
        }

        if matcher_tomb.cmd_open().is_some() {
            return open::Open::new(self.cmd_matches).invoke();
        }

        if matcher_tomb.cmd_close().is_some() {
            return close::Close::new(self.cmd_matches).invoke();
        }

        if matcher_tomb.cmd_status().is_some() {
            return status::Status::new(self.cmd_matches).invoke();
        }

        // Unreachable, clap will print help for missing sub command instead
        unreachable!()
    }
}
