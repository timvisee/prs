use clap::ArgMatches;

use crate::cmd::arg::{ArgStore, CmdArgOption};

use super::Matcher;

/// The recipients list command matcher.
pub struct ListMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> ListMatcher<'a> {
    /// The store.
    pub fn store(&self) -> String {
        ArgStore::value(self.matches)
    }
}

impl<'a> Matcher<'a> for ListMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("recipients")?
            .subcommand_matches("list")
            .map(|matches| ListMatcher { matches })
    }
}
