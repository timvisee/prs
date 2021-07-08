use anyhow::Result;
use clap::ArgMatches;
use prs_lib::Store;
use thiserror::Error;

use crate::cmd::matcher::{
    tomb::{resize::ResizeMatcher, TombMatcher},
    MainMatcher, Matcher,
};
use crate::util::{
    error,
    error::{ErrorHints, ErrorHintsBuilder},
    style,
};

/// A tomb resize action.
pub struct Resize<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Resize<'a> {
    /// Construct a new init action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the init action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_tomb = TombMatcher::with(self.cmd_matches).unwrap();
        let matcher_resize = ResizeMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_tomb.store()).map_err(Err::Store)?;
        let mut tomb = store.tomb(
            !matcher_main.verbose(),
            matcher_main.verbose(),
            matcher_main.force(),
        );

        // Must be a tomb
        if !tomb.is_tomb() && !matcher_main.force() {
            // TODO: error hint to initialize tomb
            error::quit_error_msg(
                "password store is not a tomb",
                ErrorHintsBuilder::default().force(true).build().unwrap(),
            );
        }

        // Fetch Tomb size status
        let sizes = tomb.fetch_size_stats().map_err(Err::Size)?;

        // Get size, automatically select if not given
        let size = match matcher_resize.size() {
            Some(size) => size,
            None => {
                // Get desired size
                let size = sizes.desired_tomb_size();

                // Quit if Tomb is already this big
                if let Some(tomb_size) = sizes.tomb_file_size_mbs() {
                    if tomb_size >= size {
                        eprintln!("Tomb is large enough, not resizing ({}MB)", tomb_size);
                        eprintln!(
                            "Use '{}' flag to specify a size",
                            style::highlight("--size MEGABYTE")
                        );
                        error::quit();
                    }
                }

                size
            }
        };

        // Must be closed
        let tomb_open = tomb.is_open().unwrap_or(false);
        if tomb_open && !matcher_main.force() {
            if matcher_main.verbose() {
                eprintln!("Closing Tomb...");
            }

            // Close the tomb
            tomb.close().map_err(Err::Close)?;
        }

        // New tomb size must be larger
        if let Some(tomb_file_size) = sizes.tomb_file_size_mbs() {
            if size <= tomb_file_size {
                error::quit_error_msg(
                    format!(
                        "new tomb size must be larger than current size ({}MB)",
                        tomb_file_size
                    ),
                    ErrorHints::default(),
                );
            }
        }

        // Resize tomb
        if !matcher_main.quiet() {
            eprintln!("Resizing Tomb to {}MB...", size);
        }
        tomb.resize(size).map_err(Err::Resize)?;

        // Open tomb if it was open before
        if tomb_open {
            super::open::open(&mut tomb, &matcher_main).map_err(Err::Open)?;
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to resize tomb")]
    Resize(#[source] anyhow::Error),

    #[error("failed to open tomb after resizing")]
    Open(#[source] super::open::Err),

    #[error("failed to close tomb before resizing")]
    Close(#[source] anyhow::Error),

    #[error("failed to fetch password store size status")]
    Size(#[source] anyhow::Error),
}
