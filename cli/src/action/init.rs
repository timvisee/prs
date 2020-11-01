use std::fs;
use std::path::Path;

use anyhow::Result;
use clap::ArgMatches;
use prs_lib::store::Store;
use thiserror::Error;

use crate::cmd::matcher::{init::InitMatcher, MainMatcher, Matcher};
use crate::util::{
    self,
    error::{self, ErrorHints},
    style,
};

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

        let path = shellexpand::full(&matcher_init.store())
            .map_err(Err::ExpandPath)?
            .to_string();

        ensure_dir_free(&Path::new(&path))?;

        // Initialize store
        fs::create_dir_all(&path).map_err(Err::Init)?;

        // TODO: initialize sync here?

        // Open new store
        let store = Store::open(&path).map_err(Err::Store)?;

        // Run housekeeping
        crate::action::housekeeping::run::housekeeping(&store).map_err(Err::Housekeeping)?;

        // Hint user to add our recipient key
        if !matcher_main.quiet() {
            let bin = util::bin_name();
            let system_has_secret = has_secret_key_in_keychain().unwrap_or(true);

            if system_has_secret {
                eprintln!("Now add your own key as recipient or generate a new one:");
            } else {
                eprintln!("Now generate and add a new recipient key for yourself:");
            }
            if system_has_secret {
                eprintln!(
                    "    {}",
                    style::highlight(&format!("{} recipients add --secret", bin))
                );
            }
            eprintln!(
                "    {}",
                style::highlight(&format!("{} recipients generate", bin))
            );
            eprintln!();
        }

        Ok(())
    }
}

/// Check whether the user has any secret key in his keychain.
// TODO: duplicate, also use in clone
fn has_secret_key_in_keychain() -> Result<bool> {
    Ok(!prs_lib::all(true)?.keys().is_empty())
}

/// Ensure the given path is a free directory.
///
/// Checks whether the given path is not a directory, or whehter the directory is empty.
/// Quits on error.
// TODO: duplicate in action/init, move to shared module
fn ensure_dir_free(path: &Path) -> Result<()> {
    // Fine if not a directory
    if !path.is_dir() {
        return Ok(());
    }

    // Fine if no paths in dir
    if path.read_dir().map_err(Err::Init)?.count() == 0 {
        return Ok(());
    }

    error::quit_error_msg(
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

    #[error("failed to run housekeeping tasks")]
    Housekeeping(#[source] anyhow::Error),
}
