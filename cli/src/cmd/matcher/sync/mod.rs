pub mod commit;
pub mod init;
pub mod remote;
pub mod reset;
pub mod status;

use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgAllowDirty, CmdArgFlag};

/// The sync command matcher.
pub struct SyncMatcher<'a> {
    root: &'a ArgMatches,
    matches: &'a ArgMatches,
}

impl<'a> SyncMatcher<'a> {
    /// Get the sync commit sub command, if matched.
    pub fn cmd_commit(&'a self) -> Option<commit::CommitMatcher<'a>> {
        commit::CommitMatcher::with(self.root)
    }

    /// Get the sync init sub command, if matched.
    pub fn cmd_init(&'a self) -> Option<init::InitMatcher<'a>> {
        init::InitMatcher::with(self.root)
    }

    /// Get the sync remote sub command, if matched.
    pub fn cmd_remote(&'a self) -> Option<remote::RemoteMatcher<'a>> {
        remote::RemoteMatcher::with(self.root)
    }

    /// Get the sync reset sub command, if matched.
    pub fn cmd_reset(&'a self) -> Option<reset::ResetMatcher<'a>> {
        reset::ResetMatcher::with(self.root)
    }

    /// Get the sync status sub command, if matched.
    pub fn cmd_status(&'a self) -> Option<status::StatusMatcher<'a>> {
        status::StatusMatcher::with(self.root)
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
