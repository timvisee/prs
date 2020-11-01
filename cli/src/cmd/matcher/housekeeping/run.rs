use clap::ArgMatches;

use super::Matcher;

/// The housekeeping run command matcher.
pub struct RunMatcher<'a> {
    _matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> RunMatcher<'a> {}

impl<'a> Matcher<'a> for RunMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("housekeeping")?
            .subcommand_matches("run")
            .map(|matches| RunMatcher { _matches: matches })
    }
}
