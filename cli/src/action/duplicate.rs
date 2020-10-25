use std::fs;

use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::store::Store;

use crate::cmd::matcher::{duplicate::DuplicateMatcher, MainMatcher, Matcher};
use crate::util;

/// A file duplicate action.
pub struct Duplicate<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Duplicate<'a> {
    /// Construct a new duplicate action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the duplicate action.
    // TODO: re-implement error handling
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_duplicate = DuplicateMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(crate::STORE_DEFAULT_ROOT);

        // TODO: show secret name if not equal to input, unless quiet?
        let secrets = store.secrets(matcher_duplicate.query());
        let secret = crate::select_secret(&secrets).ok_or(Err::NoneSelected)?;

        let target = matcher_duplicate.target();

        // TODO: move this into normalize function below
        let target = shellexpand::full(target).map_err(Err::ExpandTarget)?;

        // Normalize target path
        let path = store.normalize_secret_path(
            target.as_ref(),
            secret.path.file_name().and_then(|p| p.to_str()),
            true,
        );

        // Check if target already exists if not forcing
        if !matcher_main.force() && path.is_file() {
            eprintln!("A secret at '{}' already exists", path.display(),);
            if !util::prompt_yes("Overwrite?", Some(true), &matcher_main) {
                println!("Duplication cancelled");
                util::quit();
            }
        }

        // Copy secret
        fs::copy(&secret.path, path)
            .map(|_| ())
            .map_err(|err| Err::Copy(err).into())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("no secret selected")]
    NoneSelected,

    #[error("failed to expand target path")]
    ExpandTarget(#[source] shellexpand::LookupError<std::env::VarError>),

    #[error("failed to copy secret file")]
    Copy(#[source] std::io::Error),
}
