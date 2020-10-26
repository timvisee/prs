use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgStore, CmdArgOption};

/// The new command matcher.
pub struct NewMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> NewMatcher<'a> {
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

impl<'a> Matcher<'a> for NewMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("new")
            .map(|matches| NewMatcher { matches })
    }
}
