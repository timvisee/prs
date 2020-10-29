use clap::ArgMatches;

use super::Matcher;

/// The recipients add command matcher.
pub struct AddMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> AddMatcher<'a> {
    /// Check whether to skip re-encrypting secrets.
    pub fn no_recrypt(&self) -> bool {
        self.matches.is_present("no-recrypt")
    }

    /// Check whether to add a secret key.
    pub fn secret(&self) -> bool {
        self.matches.is_present("secret")
    }
}

impl<'a> Matcher<'a> for AddMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("recipients")?
            .subcommand_matches("add")
            .map(|matches| AddMatcher { matches })
    }
}
