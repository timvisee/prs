use clap::ArgMatches;

use super::Matcher;

/// The recipients remove command matcher.
pub struct RemoveMatcher<'a> {
    _matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> RemoveMatcher<'a> {}

impl<'a> Matcher<'a> for RemoveMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("recipients")?
            .subcommand_matches("remove")
            .map(|matches| RemoveMatcher { _matches: matches })
    }
}
