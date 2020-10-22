use clap::ArgMatches;

use super::Matcher;

/// The show command matcher.
pub struct ShowMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> ShowMatcher<'a> {
    /// Check whether to just show the first line of the secret.
    pub fn first_line(&self) -> bool {
        self.matches.is_present("first")
    }
}

impl<'a> Matcher<'a> for ShowMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("show")
            .map(|matches| ShowMatcher { matches })
    }
}
