use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgQuery, ArgStore, CmdArgOption};

/// The list command matcher.
pub struct ListMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> ListMatcher<'a> {
    /// The secret query.
    pub fn query(&self) -> Option<String> {
        ArgQuery::value(self.matches)
    }

    /// The store.
    pub fn store(&self) -> String {
        ArgStore::value(self.matches)
    }

    /// Whether to only show aliases.
    pub fn only_aliases(&self) -> bool {
        self.matches.is_present("aliases")
    }

    /// Whether to only show aliases.
    pub fn only_non_aliases(&self) -> bool {
        self.matches.is_present("non-aliases")
    }
}

impl<'a> Matcher<'a> for ListMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("list")
            .map(|matches| ListMatcher { matches })
    }
}
