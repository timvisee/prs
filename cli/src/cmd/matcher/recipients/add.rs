use clap::ArgMatches;

use super::Matcher;

/// The recipients add command matcher.
pub struct AddMatcher<'a> {
    _matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> AddMatcher<'a> {}

impl<'a> Matcher<'a> for AddMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("recipients")?
            .subcommand_matches("add")
            .map(|matches| AddMatcher { _matches: matches })
    }
}
