pub mod show;

use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgStore, CmdArgOption};

/// The TOTP command matcher.
pub struct TotpMatcher<'a> {
    root: &'a ArgMatches,
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> TotpMatcher<'a> {
    /// Get the TOTP show sub command, if matched.
    pub fn cmd_show(&'a self) -> Option<show::ShowMatcher> {
        show::ShowMatcher::with(self.root)
    }

    /// The store.
    pub fn store(&self) -> String {
        ArgStore::value(self.matches)
    }
}

impl<'a> Matcher<'a> for TotpMatcher<'a> {
    fn with(root: &'a ArgMatches) -> Option<Self> {
        root.subcommand_matches("totp")
            .map(|matches| TotpMatcher { root, matches })
    }
}
