use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgStore, CmdArgOption};

/// The lock command matcher.
pub struct LockMatcher<'a> {
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> LockMatcher<'a> {
    /// The store.
    pub fn store(&self) -> String {
        ArgStore::value(self.matches)
    }
}

impl<'a> Matcher<'a> for LockMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("lock")
            .map(|matches| LockMatcher { matches })
    }
}
