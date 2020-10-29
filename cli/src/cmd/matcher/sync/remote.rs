use clap::ArgMatches;

use super::Matcher;

/// The sync remote command matcher.
pub struct RemoteMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> RemoteMatcher<'a> {
    /// Get the git URL to set.
    pub fn git_url(&self) -> Option<&str> {
        self.matches.value_of("GIT_URL")
    }
}

impl<'a> Matcher<'a> for RemoteMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("sync")?
            .subcommand_matches("remote")
            .map(|matches| RemoteMatcher { matches })
    }
}
