#[cfg(feature = "clipboard")]
pub mod clip_revert;
pub mod completions;

use anyhow::Result;
use clap::ArgMatches;

use crate::cmd::matcher::{InternalMatcher, Matcher};

/// An internal action.
pub struct Internal<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Internal<'a> {
    /// Construct a new internal action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the internal action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matcher
        #[allow(unused)]
        let matcher_internal = InternalMatcher::with(self.cmd_matches).unwrap();

        #[cfg(feature = "clipboard")]
        if matcher_internal.clip_revert().is_some() {
            return clip_revert::ClipRevert::new(self.cmd_matches).invoke();
        }

        if matcher_internal.completions().is_some() {
            return completions::Completions::new(self.cmd_matches).invoke();
        }

        // Unreachable, clap will print help for missing sub command instead
        unreachable!()
    }
}
