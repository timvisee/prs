use clap::ArgMatches;

use super::Matcher;

/// The internal clipboard command matcher.
pub struct ClipMatcher<'a> {
    _matches: &'a ArgMatches,
}

impl ClipMatcher<'_> {}

impl<'a> Matcher<'a> for ClipMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("internal")?
            .subcommand_matches("clip")
            .map(|matches| ClipMatcher { _matches: matches })
    }
}
