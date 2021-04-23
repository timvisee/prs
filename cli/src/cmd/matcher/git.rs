use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgStore, CmdArgOption};

/// The git command matcher.
pub struct GitMatcher<'a> {
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> GitMatcher<'a> {
    /// Get the git command to invoke.
    pub fn command(&self) -> String {
        self.matches
            .values_of("COMMAND")
            .map(|c| c.collect::<Vec<_>>().join(" "))
            .unwrap_or_else(|| "".into())
    }

    /// The store.
    pub fn store(&self) -> String {
        ArgStore::value(self.matches)
    }
}

impl<'a> Matcher<'a> for GitMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("git")
            .map(|matches| GitMatcher { matches })
    }
}
