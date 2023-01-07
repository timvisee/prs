pub mod recrypt;
pub mod run;
pub mod sync_keys;

use clap::ArgMatches;

use super::Matcher;

/// The housekeeping matcher.
pub struct HousekeepingMatcher<'a> {
    root: &'a ArgMatches,
    _matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> HousekeepingMatcher<'a> {
    /// Get the housekepeing recrypt sub command, if matched.
    pub fn recrypt(&'a self) -> Option<recrypt::RecryptMatcher> {
        recrypt::RecryptMatcher::with(self.root)
    }

    /// Get the housekepeing run sub command, if matched.
    pub fn run(&'a self) -> Option<run::RunMatcher> {
        run::RunMatcher::with(self.root)
    }

    /// Get the housekepeing sync-keys sub command, if matched.
    pub fn sync_keys(&'a self) -> Option<sync_keys::SyncKeysMatcher> {
        sync_keys::SyncKeysMatcher::with(self.root)
    }
}

impl<'a> Matcher<'a> for HousekeepingMatcher<'a> {
    fn with(root: &'a ArgMatches) -> Option<Self> {
        root.subcommand_matches("housekeeping")
            .map(|matches| HousekeepingMatcher { root, _matches: matches })
    }
}
