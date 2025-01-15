use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, CmdArgFlag};

/// The housekeeping run command matcher.
pub struct RunMatcher<'a> {
    matches: &'a ArgMatches,
}

impl RunMatcher<'_> {
    /// Whether to allow a dirty repository for syncing.
    pub fn allow_dirty(&self) -> bool {
        ArgAllowDirty::is_present(self.matches)
    }

    /// Whether to not sync.
    pub fn no_sync(&self) -> bool {
        ArgNoSync::is_present(self.matches)
    }
}

impl<'a> Matcher<'a> for RunMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("housekeeping")?
            .subcommand_matches("run")
            .map(|matches| RunMatcher { matches })
    }
}
