use clap::ArgMatches;

use super::Matcher;

/// The sync init command matcher.
pub struct InitMatcher<'a> {
    _matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> InitMatcher<'a> {}

impl<'a> Matcher<'a> for InitMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("sync")?
            .subcommand_matches("init")
            .map(|matches| InitMatcher { _matches: matches })
    }
}
