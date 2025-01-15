use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, ArgQuery, CmdArgFlag, CmdArgOption};

/// The duplicate command matcher.
pub struct DuplicateMatcher<'a> {
    matches: &'a ArgMatches,
}

impl DuplicateMatcher<'_> {
    /// The secret query.
    pub fn query(&self) -> Option<String> {
        ArgQuery::value(self.matches)
    }

    /// Secret destination.
    pub fn destination(&self) -> &String {
        self.matches.get_one("DEST").unwrap()
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

impl<'a> Matcher<'a> for DuplicateMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("duplicate")
            .map(|matches| DuplicateMatcher { matches })
    }
}
