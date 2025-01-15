use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, CmdArgFlag};

/// The housekeeping sync-keys command matcher.
pub struct SyncKeysMatcher<'a> {
    matches: &'a ArgMatches,
}

impl SyncKeysMatcher<'_> {
    /// Check whether to not import missing keys.
    pub fn no_import(&self) -> bool {
        self.matches.get_flag("no-import")
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

impl<'a> Matcher<'a> for SyncKeysMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("housekeeping")?
            .subcommand_matches("sync-keys")
            .map(|matches| SyncKeysMatcher { matches })
    }
}
