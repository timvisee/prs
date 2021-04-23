use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgQuery, ArgStore, CmdArgOption};

/// The edit command matcher.
pub struct EditMatcher<'a> {
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> EditMatcher<'a> {
    /// The secret query.
    pub fn query(&self) -> Option<String> {
        ArgQuery::value(self.matches)
    }

    /// Check whether to read from stdin.
    pub fn stdin(&self) -> bool {
        self.matches.is_present("stdin")
    }

    /// The store.
    pub fn store(&self) -> String {
        ArgStore::value(self.matches)
    }
}

impl<'a> Matcher<'a> for EditMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("edit")
            .map(|matches| EditMatcher { matches })
    }
}
