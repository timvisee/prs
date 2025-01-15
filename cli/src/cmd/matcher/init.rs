use clap::ArgMatches;

use super::Matcher;

/// The init command matcher.
pub struct InitMatcher<'a> {
    _matches: &'a ArgMatches,
}

impl InitMatcher<'_> {}

impl<'a> Matcher<'a> for InitMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("init")
            .map(|matches| InitMatcher { _matches: matches })
    }
}
