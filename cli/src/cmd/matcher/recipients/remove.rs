use clap::ArgMatches;

use super::Matcher;

/// The recipients remove command matcher.
pub struct RemoveMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> RemoveMatcher<'a> {
    /// Check whether to re-encrypt secrets.
    pub fn recrypt(&self) -> bool {
        self.matches.is_present("recrypt")
    }
}

impl<'a> Matcher<'a> for RemoveMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("recipients")?
            .subcommand_matches("remove")
            .map(|matches| RemoveMatcher { matches })
    }
}
