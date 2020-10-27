pub mod recrypt;
pub mod sync_keys;

use clap::ArgMatches;

use crate::cmd::arg::{ArgStore, CmdArgOption};

use super::Matcher;

/// The housekeeping matcher.
pub struct HousekeepingMatcher<'a> {
    root: &'a ArgMatches<'a>,
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> HousekeepingMatcher<'a> {
    /// Get the housekepeing recrypt sub command, if matched.
    pub fn recrypt(&'a self) -> Option<recrypt::RecryptMatcher> {
        recrypt::RecryptMatcher::with(&self.root)
    }

    /// Get the housekepeing sync-keys sub command, if matched.
    pub fn sync_keys(&'a self) -> Option<sync_keys::SyncKeysMatcher> {
        sync_keys::SyncKeysMatcher::with(&self.root)
    }

    /// The store.
    pub fn store(&self) -> String {
        ArgStore::value(self.matches)
    }
}

impl<'a> Matcher<'a> for HousekeepingMatcher<'a> {
    fn with(root: &'a ArgMatches) -> Option<Self> {
        root.subcommand_matches("housekeeping")
            .map(|matches| HousekeepingMatcher { root, matches })
    }
}
