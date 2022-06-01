pub mod add;
pub mod export;
pub mod generate;
pub mod list;
pub mod remove;

use clap::ArgMatches;

use crate::cmd::arg::{ArgStore, CmdArgOption};

use super::Matcher;

/// The recipients matcher.
pub struct RecipientsMatcher<'a> {
    root: &'a ArgMatches,
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> RecipientsMatcher<'a> {
    /// Get the recipient add sub command, if matched.
    pub fn cmd_add(&'a self) -> Option<add::AddMatcher> {
        add::AddMatcher::with(self.root)
    }

    /// Get the recipient export sub command, if matched.
    pub fn cmd_export(&'a self) -> Option<export::ExportMatcher> {
        export::ExportMatcher::with(self.root)
    }

    /// Get the recipient generate sub command, if matched.
    pub fn cmd_generate(&'a self) -> Option<generate::GenerateMatcher> {
        generate::GenerateMatcher::with(self.root)
    }

    /// Get the recipient list sub command, if matched.
    pub fn cmd_list(&'a self) -> Option<list::ListMatcher> {
        list::ListMatcher::with(self.root)
    }

    /// Get the recipient remove sub command, if matched.
    pub fn cmd_remove(&'a self) -> Option<remove::RemoveMatcher> {
        remove::RemoveMatcher::with(self.root)
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
