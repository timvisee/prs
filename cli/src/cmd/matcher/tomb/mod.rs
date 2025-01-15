pub mod close;
pub mod init;
pub mod open;
pub mod resize;
pub mod status;

use clap::ArgMatches;

use super::Matcher;

/// The tomb command matcher.
pub struct TombMatcher<'a> {
    root: &'a ArgMatches,
    _matches: &'a ArgMatches,
}

impl<'a> TombMatcher<'a> {
    /// Get the tomb init sub command, if matched.
    pub fn cmd_init(&'a self) -> Option<init::InitMatcher<'a>> {
        init::InitMatcher::with(self.root)
    }

    /// Get the tomb open sub command, if matched.
    pub fn cmd_open(&'a self) -> Option<open::OpenMatcher<'a>> {
        open::OpenMatcher::with(self.root)
    }

    /// Get the tomb close sub command, if matched.
    pub fn cmd_close(&'a self) -> Option<close::CloseMatcher<'a>> {
        close::CloseMatcher::with(self.root)
    }

    /// Get the tomb status sub command, if matched.
    pub fn cmd_status(&'a self) -> Option<status::StatusMatcher<'a>> {
        status::StatusMatcher::with(self.root)
    }

    /// Get the tomb resize sub command, if matched.
    pub fn cmd_resize(&'a self) -> Option<resize::ResizeMatcher<'a>> {
        resize::ResizeMatcher::with(self.root)
    }
}

impl<'a> Matcher<'a> for TombMatcher<'a> {
    fn with(root: &'a ArgMatches) -> Option<Self> {
        root.subcommand_matches("tomb").map(|matches| TombMatcher {
            root,
            _matches: matches,
        })
    }
}
