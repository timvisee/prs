use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, CmdArgFlag};

/// The recipients add command matcher.
pub struct AddMatcher<'a> {
    matches: &'a ArgMatches,
}

impl AddMatcher<'_> {
    /// Check whether to skip re-encrypting secrets.
    pub fn no_recrypt(&self) -> bool {
        self.matches.get_flag("no-recrypt")
    }

    /// Check whether to add a secret key.
    pub fn secret(&self) -> bool {
        self.matches.get_flag("secret")
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
            .subcommand_matches("recipients")?
            .subcommand_matches("add")
            .map(|matches| AddMatcher { matches })
    }
}
