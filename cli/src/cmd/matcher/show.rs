use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgQuery, ArgStore, CmdArgOption};

/// The show command matcher.
pub struct ShowMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> ShowMatcher<'a> {
    /// Check whether to just show the first line of the secret.
    pub fn first_line(&self) -> bool {
        self.matches.is_present("first")
    }

    /// The secret query.
    pub fn query(&self) -> Option<String> {
        ArgQuery::value(self.matches)
    }

    /// The store.
    pub fn store(&self) -> String {
        ArgStore::value(self.matches)
    }
}

impl<'a> Matcher<'a> for ShowMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("show")
            .map(|matches| ShowMatcher { matches })
    }
}
