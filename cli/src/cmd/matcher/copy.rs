use clap::ArgMatches;

use super::Matcher;

/// The copy command matcher.
pub struct CopyMatcher<'a> {
    _matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> CopyMatcher<'a> {}

impl<'a> Matcher<'a> for CopyMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("copy")
            .map(|matches| CopyMatcher { _matches: matches })
    }
}
