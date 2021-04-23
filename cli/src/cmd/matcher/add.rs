use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgStore, CmdArgOption};

/// The add command matcher.
pub struct AddMatcher<'a> {
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> AddMatcher<'a> {
    /// Secret destination.
    pub fn destination(&self) -> &str {
        self.matches.value_of("DEST").unwrap()
    }

    /// Check whether to create an empty secret.
    pub fn empty(&self) -> bool {
        self.matches.is_present("empty")
    }

    /// Check whether to read from stdin.
    pub fn stdin(&self) -> bool {
        self.matches.is_present("stdin")
    }

    /// The store.
    pub fn store(&self) -> String {
        ArgStore::value(self.matches)
    }
}

impl<'a> Matcher<'a> for AddMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("add")
            .map(|matches| AddMatcher { matches })
    }
}
