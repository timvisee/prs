use clap::ArgMatches;

use super::Matcher;

/// The show command matcher.
pub struct ShowMatcher<'a> {
    _matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> ShowMatcher<'a> {}

impl<'a> Matcher<'a> for ShowMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("show")
            .map(|matches| ShowMatcher { _matches: matches })
    }
}
