pub mod add;
pub mod export;
pub mod generate;
pub mod list;
pub mod remove;

use clap::ArgMatches;

use super::Matcher;

/// The recipients matcher.
pub struct RecipientsMatcher<'a> {
    root: &'a ArgMatches,
    _matches: &'a ArgMatches,
}

impl<'a> RecipientsMatcher<'_> {
    /// Get the recipient add sub command, if matched.
    pub fn cmd_add(&'a self) -> Option<add::AddMatcher<'a>> {
        add::AddMatcher::with(self.root)
    }

    /// Get the recipient export sub command, if matched.
    pub fn cmd_export(&'a self) -> Option<export::ExportMatcher<'a>> {
        export::ExportMatcher::with(self.root)
    }

    /// Get the recipient generate sub command, if matched.
    pub fn cmd_generate(&'a self) -> Option<generate::GenerateMatcher<'a>> {
        generate::GenerateMatcher::with(self.root)
    }

    /// Get the recipient list sub command, if matched.
    pub fn cmd_list(&'a self) -> Option<list::ListMatcher<'a>> {
        list::ListMatcher::with(self.root)
    }

    /// Get the recipient remove sub command, if matched.
    pub fn cmd_remove(&'a self) -> Option<remove::RemoveMatcher<'a>> {
        remove::RemoveMatcher::with(self.root)
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
