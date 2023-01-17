use std::io;

use anyhow::Result;
use clap::ArgMatches;
use prs_lib::Plaintext;
use thiserror::Error;

use crate::cmd::matcher::{internal::clip::ClipMatcher, MainMatcher, Matcher};
use crate::util::{base64, clipboard};

/// A internal clipboard action.
pub struct Clip<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Clip<'a> {
    /// Construct a new clipboard action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the clipboard action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let _matcher_clip = ClipMatcher::with(self.cmd_matches).unwrap();

        // Grab clipboard data from stdin
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;
        let data = base64::decode(buffer.trim()).map_err(Err::Data)?.into();
        drop(Plaintext::from(buffer));

        // Set clipboard contents
        clipboard::subprocess_copy(&data, matcher_main.quiet(), matcher_main.verbose())
            .map_err(Err::Clip)?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to obtain clipboard content from stdin, malformed data")]
    Data(#[source] ::base64::DecodeError),

    #[error("failed to set clipboard contents")]
    Clip(#[source] anyhow::Error),
}
