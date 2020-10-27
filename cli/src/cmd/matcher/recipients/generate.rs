use clap::ArgMatches;

use super::Matcher;

/// The recipients generate command matcher.
pub struct GenerateMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> GenerateMatcher<'a> {
    /// Check whether to skip adding key to store.
    pub fn skip_add(&self) -> bool {
        self.matches.is_present("skip-add")
    }
}

impl<'a> Matcher<'a> for GenerateMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("recipients")?
            .subcommand_matches("generate")
            .map(|matches| GenerateMatcher { matches })
    }
}
