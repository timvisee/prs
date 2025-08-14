use std::io;
use std::time::Duration;

use anyhow::Result;
use clap::ArgMatches;
use prs_lib::Plaintext;
use thiserror::Error;

use crate::cmd::matcher::{MainMatcher, Matcher, internal::clip_revert::ClipRevertMatcher};
use crate::util::{base64, clipboard};

/// A internal clipboard revert action.
pub struct ClipRevert<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> ClipRevert<'a> {
    /// Construct a new clipboard revert action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the clipboard revert action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_clip_revert = ClipRevertMatcher::with(self.cmd_matches).unwrap();

        // Grab clipboard data from stdin
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;
        let (a, b) = buffer.split_once(',').ok_or(Err::Data(None))?;
        let (data, data_old) = (
            base64::decode(a.trim())
                .map_err(|err| Err::Data(Some(err)))?
                .into(),
            base64::decode(b.trim())
                .map_err(|err| Err::Data(Some(err)))?
                .into(),
        );
        drop(Plaintext::from(buffer));

        let timeout = Duration::from_secs(matcher_clip_revert.timeout().unwrap());

        // Set clipboard contents
        clipboard::subprocess_copy_revert(
            &data,
            &data_old,
            timeout,
            matcher_main.quiet(),
            matcher_main.verbose(),
        )
        .map_err(Err::CopyRevert)?;

        if matcher_main.verbose() {
            eprintln!("Clipboard reverted");
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to obtain clipboard content from stdin, malformed data")]
    Data(#[source] Option<::base64::DecodeError>),

    #[error("failed to copy and revert clipboard contents")]
    CopyRevert(#[source] anyhow::Error),
}
