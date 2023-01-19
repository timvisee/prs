use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgNoSync, CmdArgFlag};

/// The sync commit command matcher.
pub struct CommitMatcher<'a> {
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> CommitMatcher<'a> {
    /// Whether to not sync.
    pub fn no_sync(&self) -> bool {
        ArgNoSync::is_present(self.matches)
    }
}

impl<'a> Matcher<'a> for CommitMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("sync")?
            .subcommand_matches("commit")
            .map(|matches| CommitMatcher { matches })
    }
}
