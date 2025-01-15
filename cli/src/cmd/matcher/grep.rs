use clap::ArgMatches;

use super::Matcher;

/// The grep command matcher.
pub struct GrepMatcher<'a> {
    matches: &'a ArgMatches,
}

impl GrepMatcher<'_> {
    /// The grep pattern.
    pub fn pattern(&self) -> String {
        self.matches.get_one("PATTERN").cloned().unwrap()
    }

    /// The secret query.
    pub fn query(&self) -> Option<String> {
        self.matches.get_one("query").cloned()
    }

    /// Whether to parse the pattern as regular expression.
    pub fn regex(&self) -> bool {
        self.matches.get_flag("regex")
    }

    /// Whether to include searching aliases.
    pub fn with_aliases(&self) -> bool {
        self.matches.get_flag("aliases")
    }
}

impl<'a> Matcher<'a> for GrepMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("grep")
            .map(|matches| GrepMatcher { matches })
    }
}
