use clap::ArgMatches;

use super::Matcher;

/// The tomb close command matcher.
pub struct CloseMatcher<'a> {
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> CloseMatcher<'a> {
    /// Whether to try to close.
    pub fn do_try(&self) -> bool {
        self.matches.get_flag("try")
    }
}

impl<'a> Matcher<'a> for CloseMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("tomb")?
            .subcommand_matches("close")
            .map(|matches| CloseMatcher { matches })
    }
}
