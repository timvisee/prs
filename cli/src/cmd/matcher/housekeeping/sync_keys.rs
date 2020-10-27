use clap::ArgMatches;

use super::Matcher;

/// The housekeeping sync-keys command matcher.
pub struct SyncKeysMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> SyncKeysMatcher<'a> {
    /// Check whether to not import missing keys.
    pub fn no_import(&self) -> bool {
        self.matches.is_present("no-import")
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
