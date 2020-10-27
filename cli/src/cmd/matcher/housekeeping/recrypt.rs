use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgQuery, CmdArgOption};

/// The housekeeping recrypt command matcher.
pub struct RecryptMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> RecryptMatcher<'a> {
    /// The secret query.
    pub fn query(&self) -> Option<String> {
        ArgQuery::value(self.matches)
    }

    pub fn _all(&self) -> bool {
        self.matches.is_present("all")
    }
}

impl<'a> Matcher<'a> for RecryptMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("housekeeping")?
            .subcommand_matches("recrypt")
            .map(|matches| RecryptMatcher { matches })
    }
}
