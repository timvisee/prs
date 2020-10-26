use std::fs;
use std::path::Path;

use anyhow::Result;
use clap::ArgMatches;
use prs_lib::store::Store;
use thiserror::Error;

use crate::cmd::matcher::{init::InitMatcher, MainMatcher, Matcher};
use crate::util::{self, ErrorHints};

/// Init store action.
pub struct Init<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Init<'a> {
    /// Construct a new init action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the init action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_init = InitMatcher::with(self.cmd_matches).unwrap();

        let path = shellexpand::full(matcher_init.path()).map_err(Err::ExpandPath)?;

        ensure_dir_free(&Path::new(path.as_ref()))?;

        // Initialize store
        fs::create_dir_all(path.as_ref()).map_err(Err::Init)?;

        // TODO: assign store recipients

        // TODO: initialize sync with git

        // Open the store to test
        let store = Store::open(path.as_ref()).map_err(Err::Store)?;

        // Use all keyring recipients by default, write to store
        let recipients = prs_lib::all()?;
        recipients.write_to_file(store.gpg_ids_file())?;

        // TODO: also write public keys to store

        if !matcher_main.quiet() {
            eprintln!("Store initialized");
        }

        Ok(())
    }
}

/// Ensure the given path is a free directory.
///
/// Checks whether the given path is not a directory, or whehter the directory is empty.
/// Quits on error.
fn ensure_dir_free(path: &Path) -> Result<()> {
    // Fine if not a directory
    if !path.is_dir() {
        return Ok(());
    }

    // Fine if no paths in dir
    if path.read_dir().map_err(Err::Init)?.count() == 0 {
        return Ok(());
    }

    util::quit_error_msg(
        format!(
            "cannot initialize store, directory already exists: {}",
            path.display(),
        ),
        ErrorHints::default(),
    )
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to expand store path")]
    ExpandPath(#[source] shellexpand::LookupError<std::env::VarError>),

    #[error("failed to initialize store")]
    Init(#[source] std::io::Error),

    #[error("failed to access initialized password store")]
    Store(#[source] anyhow::Error),
}
