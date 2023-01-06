use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgStore, CmdArgOption};

/// The slam command matcher.
pub struct SlamMatcher<'a> {
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> SlamMatcher<'a> {
    /// The store.
    pub fn store(&self) -> String {
        ArgStore::value(self.matches)
    }
}

impl<'a> Matcher<'a> for SlamMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("slam")
            .map(|matches| SlamMatcher { matches })
    }
}
