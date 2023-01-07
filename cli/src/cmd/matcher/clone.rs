use clap::ArgMatches;

use super::Matcher;

/// The clone command matcher.
pub struct CloneMatcher<'a> {
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> CloneMatcher<'a> {
    /// The git URL to clone from.
    pub fn git_url(&self) -> &String {
        self.matches.get_one("GIT_URL").unwrap()
    }
}

impl<'a> Matcher<'a> for CloneMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("clone")
            .map(|matches| CloneMatcher { matches })
    }
}
