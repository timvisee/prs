use clap::ArgMatches;

use super::Matcher;

/// The tomb close command matcher.
pub struct CloseMatcher<'a> {
    _matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> CloseMatcher<'a> {}

impl<'a> Matcher<'a> for CloseMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("tomb")?
            .subcommand_matches("close")
            .map(|matches| CloseMatcher { _matches: matches })
    }
}
