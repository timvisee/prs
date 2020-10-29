use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgStore, CmdArgOption};

/// The clone command matcher.
pub struct CloneMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> CloneMatcher<'a> {
    /// The git URL to clone from.
    pub fn git_url(&self) -> &str {
        self.matches.value_of("GIT_URL").unwrap()
    }

    /// The store.
    pub fn store(&self) -> String {
        ArgStore::value(self.matches)
    }
}

impl<'a> Matcher<'a> for CloneMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("clone")
            .map(|matches| CloneMatcher { matches })
    }
}
