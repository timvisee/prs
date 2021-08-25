use clap::ArgMatches;

use super::Matcher;

/// The main command matcher.
pub struct MainMatcher<'a> {
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> MainMatcher<'a> {
    /// Check whether to force.
    pub fn force(&self) -> bool {
        self.matches.is_present("force")
    }

    /// Check whether to use no-interact mode.
    pub fn no_interact(&self) -> bool {
        self.matches.is_present("no-interact")
    }

    /// Check whether to assume yes.
    pub fn assume_yes(&self) -> bool {
        self.matches.is_present("yes")
    }

    /// Check whether quiet mode is used.
    pub fn quiet(&self) -> bool {
        !self.verbose() && self.matches.is_present("quiet")
    }

    /// Check whether verbose mode is used.
    pub fn verbose(&self) -> bool {
        self.matches.is_present("verbose")
    }

    /// Check whether to use GPG in TTY mode.
    pub fn gpg_tty(&self) -> bool {
        self.matches.is_present("gpg-tty")
    }
}

impl<'a> Matcher<'a> for MainMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        Some(MainMatcher { matches })
    }
}
