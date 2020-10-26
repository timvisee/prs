pub mod list;

use clap::ArgMatches;

use super::Matcher;
use list::ListMatcher;

/// The recipients matcher.
pub struct RecipientsMatcher<'a> {
    root: &'a ArgMatches<'a>,
    _matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> RecipientsMatcher<'a> {
    /// Get the recipient list sub command, if matched.
    pub fn list(&'a self) -> Option<ListMatcher> {
        ListMatcher::with(&self.root)
    }
}

impl<'a> Matcher<'a> for RecipientsMatcher<'a> {
    fn with(root: &'a ArgMatches) -> Option<Self> {
        root.subcommand_matches("recipients")
            .map(|matches| RecipientsMatcher {
                root,
                _matches: matches,
            })
    }
}
