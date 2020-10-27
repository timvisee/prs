pub mod add;
pub mod list;

use clap::ArgMatches;

use crate::cmd::arg::{ArgStore, CmdArgOption};

use super::Matcher;

/// The recipients matcher.
pub struct RecipientsMatcher<'a> {
    root: &'a ArgMatches<'a>,
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> RecipientsMatcher<'a> {
    /// Get the recipient add sub command, if matched.
    pub fn add(&'a self) -> Option<add::AddMatcher> {
        add::AddMatcher::with(&self.root)
    }

    /// Get the recipient list sub command, if matched.
    pub fn list(&'a self) -> Option<list::ListMatcher> {
        list::ListMatcher::with(&self.root)
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
