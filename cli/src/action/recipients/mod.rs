pub mod add;
pub mod list;
pub mod remove;

use anyhow::Result;
use clap::ArgMatches;

use crate::cmd::matcher::{Matcher, RecipientsMatcher};

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
    pub fn invoke(&self) -> Result<()> {
        // Create the command matcher
        let matcher_recipients = RecipientsMatcher::with(self.cmd_matches).unwrap();

        if matcher_recipients.add().is_some() {
            return add::Add::new(self.cmd_matches).invoke();
        }

        if matcher_recipients.list().is_some() {
            return list::List::new(self.cmd_matches).invoke();
        }

        if matcher_recipients.remove().is_some() {
            return remove::Remove::new(self.cmd_matches).invoke();
        }

        // Unreachable, clap will print help for missing sub command instead
        unreachable!()
    }
}
