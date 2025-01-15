use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgProperty, ArgQuery, CmdArgOption};

/// The TOTP live command matcher.
pub struct LiveMatcher<'a> {
    matches: &'a ArgMatches,
}

impl LiveMatcher<'_> {
    /// Check whether to follow.
    pub fn follow(&self) -> bool {
        self.matches.get_flag("follow")
    }

    /// The secret query.
    pub fn query(&self) -> Option<String> {
        ArgQuery::value(self.matches)
    }

    /// The selected property.
    pub fn property(&self) -> Option<&String> {
        ArgProperty::value(self.matches)
    }
}

impl<'a> Matcher<'a> for LiveMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("totp")?
            .subcommand_matches("live")
            .map(|matches| LiveMatcher { matches })
    }
}
