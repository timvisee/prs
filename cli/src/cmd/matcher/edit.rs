use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, ArgQuery, CmdArgFlag, CmdArgOption};

/// The edit command matcher.
pub struct EditMatcher<'a> {
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> EditMatcher<'a> {
    /// The secret query.
    pub fn query(&self) -> Option<String> {
        ArgQuery::value(self.matches)
    }

    /// Check whether to read from stdin.
    pub fn stdin(&self) -> bool {
        self.matches.get_flag("stdin")
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

impl<'a> Matcher<'a> for EditMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("edit")
            .map(|matches| EditMatcher { matches })
    }
}
