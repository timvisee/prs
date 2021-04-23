pub mod add;
pub mod export;
pub mod generate;
pub mod list;
pub mod remove;

use anyhow::Result;
use clap::ArgMatches;

use crate::cmd::matcher::{Matcher, RecipientsMatcher};

/// A file recipients action.
pub struct Recipients<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Recipients<'a> {
    /// Construct a new recipients action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the recipients action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matcher
        let matcher_recipients = RecipientsMatcher::with(self.cmd_matches).unwrap();

        if matcher_recipients.cmd_add().is_some() {
            return add::Add::new(self.cmd_matches).invoke();
        }

        if matcher_recipients.cmd_export().is_some() {
            return export::Export::new(self.cmd_matches).invoke();
        }

        if matcher_recipients.cmd_generate().is_some() {
            return generate::Generate::new(self.cmd_matches).invoke();
        }

        if matcher_recipients.cmd_list().is_some() {
            return list::List::new(self.cmd_matches).invoke();
        }

        if matcher_recipients.cmd_remove().is_some() {
            return remove::Remove::new(self.cmd_matches).invoke();
        }

        // Unreachable, clap will print help for missing sub command instead
        unreachable!()
    }
}
