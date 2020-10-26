use clap::ArgMatches;

use super::Matcher;

/// The init command matcher.
pub struct InitMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> InitMatcher<'a> {
    /// Store path.
    pub fn path(&self) -> &str {
        self.matches.value_of("PATH").unwrap()
    }
}

impl<'a> Matcher<'a> for InitMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("init")
            .map(|matches| InitMatcher { matches })
    }
}
