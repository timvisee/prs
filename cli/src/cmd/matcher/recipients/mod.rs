pub mod list;

use clap::ArgMatches;

use crate::cmd::arg::{ArgStore, CmdArgOption};

use super::Matcher;
use list::ListMatcher;

/// The recipients matcher.
pub struct RecipientsMatcher<'a> {
    root: &'a ArgMatches<'a>,
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> RecipientsMatcher<'a> {
    /// Get the recipient list sub command, if matched.
    pub fn list(&'a self) -> Option<ListMatcher> {
        ListMatcher::with(&self.root)
    }

    /// The store.
    pub fn store(&self) -> String {
        ArgStore::value(self.matches)
    }
}

impl<'a> Matcher<'a> for RecipientsMatcher<'a> {
    fn with(root: &'a ArgMatches) -> Option<Self> {
        root.subcommand_matches("recipients")
            .map(|matches| RecipientsMatcher { root, matches })
    }
}
