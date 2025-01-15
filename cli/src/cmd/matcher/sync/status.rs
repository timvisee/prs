use clap::ArgMatches;

use super::Matcher;

/// The sync status command matcher.
pub struct StatusMatcher<'a> {
    _matches: &'a ArgMatches,
}

impl StatusMatcher<'_> {}

impl<'a> Matcher<'a> for StatusMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("sync")?
            .subcommand_matches("status")
            .map(|matches| StatusMatcher { _matches: matches })
    }
}
