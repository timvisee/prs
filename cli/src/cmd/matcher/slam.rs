use clap::ArgMatches;

use super::Matcher;

/// The slam command matcher.
pub struct SlamMatcher<'a> {
    _matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> SlamMatcher<'a> { }

impl<'a> Matcher<'a> for SlamMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("slam")
            .map(|matches| SlamMatcher { _matches: matches })
    }
}
