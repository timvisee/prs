#[cfg(feature = "clipboard")]
pub mod copy;
pub mod live;
pub mod show;
pub mod qr;

use anyhow::Result;
use clap::ArgMatches;

use crate::cmd::matcher::{totp::TotpMatcher, Matcher};

/// TOTP action.
pub struct Totp<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Totp<'a> {
    /// Construct a new sync action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the sync action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_totp = TotpMatcher::with(self.cmd_matches).unwrap();

        #[cfg(feature = "clipboard")]
        if matcher_totp.cmd_copy().is_some() {
            return copy::Copy::new(self.cmd_matches).invoke();
        }

        if matcher_totp.cmd_live().is_some() {
            return live::Live::new(self.cmd_matches).invoke();
        }

        if matcher_totp.cmd_qr().is_some() {
            return qr::Qr::new(self.cmd_matches).invoke();
        }

        if matcher_totp.cmd_show().is_some() {
            return show::Show::new(self.cmd_matches).invoke();
        }

        // Unreachable, clap will print help for missing sub command instead
        unreachable!()
    }
}
