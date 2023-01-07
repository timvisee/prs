use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgQuery, CmdArgOption};

/// The list command matcher.
pub struct ListMatcher<'a> {
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> ListMatcher<'a> {
    /// The secret query.
    pub fn query(&self) -> Option<String> {
        ArgQuery::value(self.matches)
    }

    /// Whether to show as plain list.
    pub fn list(&self) -> bool {
        self.matches.get_flag("list")
    }

    /// Whether to only show aliases.
    pub fn only_aliases(&self) -> bool {
        self.matches.get_flag("aliases")
    }

    /// Whether to only show aliases.
    pub fn only_non_aliases(&self) -> bool {
        self.matches.get_flag("non-aliases")
    }
}

impl<'a> Matcher<'a> for ListMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("list")
            .map(|matches| ListMatcher { matches })
    }
}
