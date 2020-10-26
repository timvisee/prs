use clap::ArgMatches;

use super::Matcher;

/// The new command matcher.
pub struct NewMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> NewMatcher<'a> {
    /// Secret destination.
    pub fn destination(&self) -> &str {
        self.matches.value_of("DEST").unwrap()
    }
}

impl<'a> Matcher<'a> for NewMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("new")
            .map(|matches| NewMatcher { matches })
    }
}
