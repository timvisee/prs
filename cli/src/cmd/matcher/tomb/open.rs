use clap::ArgMatches;

use super::Matcher;

/// The tomb open command matcher.
pub struct OpenMatcher<'a> {
    _matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> OpenMatcher<'a> {}

impl<'a> Matcher<'a> for OpenMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("tomb")?
            .subcommand_matches("open")
            .map(|matches| OpenMatcher { _matches: matches })
    }
}
