use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, CmdArgFlag};

/// The recipients remove command matcher.
pub struct RemoveMatcher<'a> {
    matches: &'a ArgMatches,
}

impl RemoveMatcher<'_> {
    /// Check whether to re-encrypt secrets.
    pub fn recrypt(&self) -> bool {
        self.matches.get_flag("recrypt")
    }

    /// Whether to allow a dirty repository for syncing.
    pub fn allow_dirty(&self) -> bool {
        ArgAllowDirty::is_present(self.matches)
    }

    /// Whether to not sync.
    pub fn no_sync(&self) -> bool {
        ArgNoSync::is_present(self.matches)
    }
}

impl<'a> Matcher<'a> for RemoveMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("recipients")?
            .subcommand_matches("remove")
            .map(|matches| RemoveMatcher { matches })
    }
}
