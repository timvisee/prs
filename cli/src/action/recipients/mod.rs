pub mod list;

use anyhow::Result;
use clap::ArgMatches;

use crate::cmd::matcher::{Matcher, RecipientsMatcher};
use list::List;

/// A file recipients action.
pub struct Recipients<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Recipients<'a> {
    /// Construct a new recipients action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the recipients action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<()> {
        // Create the command matcher
        let matcher_recipients = RecipientsMatcher::with(self.cmd_matches).unwrap();

        if matcher_recipients.list().is_some() {
            return List::new(self.cmd_matches).invoke();
        }

        // Unreachable, clap will print help for missing sub command instead
        unreachable!()
    }
}
