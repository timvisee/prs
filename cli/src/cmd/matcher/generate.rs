use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgStore, CmdArgOption};

/// The generate command matcher.
pub struct GenerateMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> GenerateMatcher<'a> {
    /// Secret destination.
    pub fn destination(&self) -> &str {
        self.matches.value_of("DEST").unwrap()
    }

    /// Check whether to edit the secret.
    pub fn edit(&self) -> bool {
        self.matches.is_present("edit")
    }

    /// Check whether to read from stdin.
    pub fn stdin(&self) -> bool {
        self.matches.is_present("stdin")
    }

    /// Check whether to read from copy.
    pub fn copy(&self) -> bool {
        self.matches.is_present("copy")
    }

    /// Check whether to read from show.
    pub fn show(&self) -> bool {
        self.matches.is_present("show")
    }

    /// The store.
    pub fn store(&self) -> String {
        ArgStore::value(self.matches)
    }
}

impl<'a> Matcher<'a> for GenerateMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("generate")
            .map(|matches| GenerateMatcher { matches })
    }
}
