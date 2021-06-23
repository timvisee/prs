use clap::ArgMatches;

use super::Matcher;

/// The tomb status command matcher.
pub struct StatusMatcher<'a> {
    _matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> StatusMatcher<'a> {}

impl<'a> Matcher<'a> for StatusMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("tomb")?
            .subcommand_matches("status")
            .map(|matches| StatusMatcher { _matches: matches })
    }
}
