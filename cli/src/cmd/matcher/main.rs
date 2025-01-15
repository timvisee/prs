use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgStore, CmdArgOption};

/// The main command matcher.
pub struct MainMatcher<'a> {
    matches: &'a ArgMatches,
}

impl MainMatcher<'_> {
    /// Check whether to force.
    pub fn force(&self) -> bool {
        self.matches.get_flag("force")
    }

    /// Check whether to use no-interact mode.
    pub fn no_interact(&self) -> bool {
        self.matches.get_flag("no-interact")
    }

    /// Check whether to assume yes.
    pub fn assume_yes(&self) -> bool {
        self.matches.get_flag("yes")
    }

    /// Check whether quiet mode is used.
    pub fn quiet(&self) -> bool {
        !self.verbose() && self.matches.get_flag("quiet")
    }

    /// Check whether verbose mode is used.
    pub fn verbose(&self) -> bool {
        self.matches.get_count("verbose") > 0
    }

    /// The store.
    pub fn store(&self) -> String {
        ArgStore::value(self.matches)
    }

    /// Check whether to use GPG in TTY mode.
    pub fn gpg_tty(&self) -> bool {
        self.matches.get_flag("gpg-tty")
    }
}

impl<'a> Matcher<'a> for MainMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        Some(MainMatcher { matches })
    }
}
