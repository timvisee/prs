use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgNoSync, CmdArgFlag};

/// The sync reset command matcher.
pub struct ResetMatcher<'a> {
    matches: &'a ArgMatches,
}

impl ResetMatcher<'_> {
    /// Whether to not sync.
    pub fn no_sync(&self) -> bool {
        ArgNoSync::is_present(self.matches)
    }
}

impl<'a> Matcher<'a> for ResetMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("sync")?
            .subcommand_matches("reset")
            .map(|matches| ResetMatcher { matches })
    }
}
