use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, CmdArgFlag};

/// The add command matcher.
pub struct AddMatcher<'a> {
    matches: &'a ArgMatches,
}

impl AddMatcher<'_> {
    /// Secret destination.
    pub fn name(&self) -> &String {
        self.matches.get_one("NAME").unwrap()
    }

    /// Check whether to create an empty secret.
    pub fn empty(&self) -> bool {
        self.matches.get_flag("empty")
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

impl<'a> Matcher<'a> for AddMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("add")
            .map(|matches| AddMatcher { matches })
    }
}
