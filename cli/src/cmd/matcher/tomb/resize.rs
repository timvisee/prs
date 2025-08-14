use anyhow::anyhow;
use clap::ArgMatches;

use super::Matcher;
use crate::util::error::{ErrorHints, quit_error, quit_error_msg};

/// The tomb resize command matcher.
pub struct ResizeMatcher<'a> {
    matches: &'a ArgMatches,
}

impl ResizeMatcher<'_> {
    /// The size in megabytes.
    pub fn size(&self) -> Option<u32> {
        let size: &String = self.matches.get_one("size")?;
        let size = match size.parse::<u32>() {
            Ok(size) => size,
            Err(err) => quit_error(
                anyhow!(err).context("invalid tomb size"),
                ErrorHints::default(),
            ),
        };

        // Size must be at least 10
        if size < 10 {
            quit_error_msg("tomb size must be at least 10MB", ErrorHints::default());
        }

        Some(size)
    }
}

impl<'a> Matcher<'a> for ResizeMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("tomb")?
            .subcommand_matches("resize")
            .map(|matches| ResizeMatcher { matches })
    }
}
