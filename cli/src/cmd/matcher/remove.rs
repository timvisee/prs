use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgQuery, ArgStore, CmdArgOption};

/// The remove command matcher.
pub struct RemoveMatcher<'a> {
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> RemoveMatcher<'a> {
    /// The secret query.
    pub fn query(&self) -> Option<String> {
        ArgQuery::value(self.matches)
    }

    /// The store.
    pub fn store(&self) -> String {
        ArgStore::value(self.matches)
    }
}

impl<'a> Matcher<'a> for RemoveMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("remove")
            .map(|matches| RemoveMatcher { matches })
    }
}
