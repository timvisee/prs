use clap::{parser::ValuesRef, ArgMatches};

use super::Matcher;

/// The git command matcher.
pub struct GitMatcher<'a> {
    matches: &'a ArgMatches,
}

impl GitMatcher<'_> {
    /// Get the git command to invoke.
    pub fn command(&self) -> String {
        self.matches
            .get_many("COMMAND")
            .map(|c: ValuesRef<String>| c.map(|s| s.as_str()).collect::<Vec<_>>().join(" "))
            .unwrap_or_default()
    }
}

impl<'a> Matcher<'a> for GitMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("git")
            .map(|matches| GitMatcher { matches })
    }
}
