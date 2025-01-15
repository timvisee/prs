use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgNoSync, CmdArgFlag};

/// The sync commit command matcher.
pub struct CommitMatcher<'a> {
    matches: &'a ArgMatches,
}

impl CommitMatcher<'_> {
    /// Whether to not sync.
    pub fn no_sync(&self) -> bool {
        ArgNoSync::is_present(self.matches)
    }

    /// Custom commit message.
    pub fn message(&self) -> Option<&str> {
        self.matches.get_one("message").map(|s: &String| s.as_str())
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
