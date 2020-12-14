#[cfg(feature = "clipboard")]
pub mod clip_revert;

use anyhow::Result;
use clap::ArgMatches;

use crate::cmd::matcher::{InternalMatcher, Matcher};

/// An internal action.
pub struct Internal<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Internal<'a> {
    /// Construct a new internal action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the internal action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matcher
        let matcher_internal = InternalMatcher::with(self.cmd_matches).unwrap();

        #[cfg(feature = "clipboard")]
        {
            if matcher_internal.clip_revert().is_some() {
                return clip_revert::ClipRevert::new(self.cmd_matches).invoke();
            }
        }

        // Unreachable, clap will print help for missing sub command instead
        unreachable!()
    }
}
