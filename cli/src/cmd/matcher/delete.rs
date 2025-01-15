use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgQuery, CmdArgOption};

/// The delete command matcher.
pub struct DeleteMatcher<'a> {
    matches: &'a ArgMatches,
}

impl DeleteMatcher<'_> {
    /// The secret query.
    pub fn query(&self) -> Option<String> {
        ArgQuery::value(self.matches)
    }
}

impl<'a> Matcher<'a> for DeleteMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("delete")
            .map(|matches| DeleteMatcher { matches })
    }
}
