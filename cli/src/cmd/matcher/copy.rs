use clap::ArgMatches;

use super::Matcher;

/// The copy command matcher.
pub struct CopyMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> CopyMatcher<'a> {
    /// Check whether to copy all of the secret.
    pub fn all(&self) -> bool {
        self.matches.is_present("all")
    }
}

impl<'a> Matcher<'a> for CopyMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("copy")
            .map(|matches| CopyMatcher { matches })
    }
}
