use std::fs;

use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::store::Store;

use crate::cmd::matcher::{r#move::MoveMatcher, MainMatcher, Matcher};
use crate::util;

/// Move secret action.
pub struct Move<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Move<'a> {
    /// Construct a new move action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the move action.
    // TODO: re-implement error handling
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_move = MoveMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(crate::STORE_DEFAULT_ROOT).map_err(Err::Store)?;

        // TODO: show secret name if not equal to input, unless quiet?
        let secrets = store.secrets(matcher_move.query());
        let secret = crate::select_secret(&secrets).ok_or(Err::NoneSelected)?;

        let dest = matcher_move.destination();

        // Normalize destination path
        let path = store
            .normalize_secret_path(dest, secret.path.file_name().and_then(|p| p.to_str()), true)
            .map_err(Err::NormalizePath)?;

        // Check if destination already exists if not forcing
        if !matcher_main.force() && path.is_file() {
            eprintln!("A secret at '{}' already exists", path.display(),);
            if !util::prompt_yes("Overwrite?", Some(true), &matcher_main) {
                if matcher_main.verbose() {
                    eprintln!("Move cancelled");
                }
                util::quit();
            }
        }

        // Move secret
        fs::rename(&secret.path, path)
            .map(|_| ())
            .map_err(|err| Err::Move(err))?;

        if !matcher_main.quiet() {
            eprintln!("Secret moved");
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("no secret selected")]
    NoneSelected,

    #[error("failed to normalize destination path")]
    NormalizePath(#[source] anyhow::Error),

    #[error("failed to move secret file")]
    Move(#[source] std::io::Error),
}
