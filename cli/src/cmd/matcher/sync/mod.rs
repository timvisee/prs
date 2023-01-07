pub mod init;
pub mod remote;

use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgAllowDirty, CmdArgFlag};

/// The sync command matcher.
pub struct SyncMatcher<'a> {
    root: &'a ArgMatches,
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> SyncMatcher<'a> {
    /// Get the sync init sub command, if matched.
    pub fn cmd_init(&'a self) -> Option<init::InitMatcher> {
        init::InitMatcher::with(self.root)
    }

    /// Get the sync remote sub command, if matched.
    pub fn cmd_remote(&'a self) -> Option<remote::RemoteMatcher> {
        remote::RemoteMatcher::with(self.root)
    }

    /// Whether to allow a dirty repository for syncing.
    pub fn allow_dirty(&self) -> bool {
        ArgAllowDirty::is_present(self.matches)
    }
}

impl<'a> Matcher<'a> for SyncMatcher<'a> {
    fn with(root: &'a ArgMatches) -> Option<Self> {
        root.subcommand_matches("sync")
            .map(|matches| SyncMatcher { root, matches })
    }
}
