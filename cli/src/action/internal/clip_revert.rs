use std::io;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use crate::cmd::matcher::{internal::clip_revert::ClipRevertMatcher, MainMatcher, Matcher};
use crate::util::clipboard;

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

        // Remember current clipboard contents, fetch previous contents
        let current = clipboard::get().map_err(Err::Current)?;
        let mut previous = None;
        if matcher_clip_revert.previous_base64_stdin() {
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer)?;
            previous = base64::decode(buffer.trim()).ok();
        }

        // Wait for timeout
        let timeout = matcher_clip_revert.timeout().unwrap();
        if timeout > 0 {
            thread::sleep(Duration::from_secs(timeout));
        }

        // Revert clipboard to previous if contents didn't change
        if current == clipboard::get().map_err(Err::Current)? {
            clipboard::set(previous.as_deref().unwrap_or(&[])).map_err(Err::Revert)?;
        }

        if matcher_main.verbose() {
            eprintln!("Clipboard cleared");
        }

        // Notify user about cleared clipboard
        clipboard::notify_cleared().map_err(Err::Notify)?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to get current clipboard contents")]
    Current(#[source] anyhow::Error),

    #[error("failed to revert clipboard")]
    Revert(#[source] anyhow::Error),

    #[error("failed to notify user for cleared clipboard")]
    Notify(#[source] anyhow::Error),
}
