use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{
    crypto::{self, prelude::*},
    Plaintext, Secret, Store,
};
use thiserror::Error;

use crate::cmd::matcher::{add::AddMatcher, MainMatcher, Matcher};
use crate::util::{cli, edit, error, stdin, sync};

/// Add secret action.
pub struct Add<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Add<'a> {
    /// Construct a new add action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the add action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_add = AddMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_add.store()).map_err(Err::Store)?;
        let sync = store.sync();
        let dest = matcher_add.destination();

        sync::ensure_ready(&sync);
        sync.prepare()?;

        // Normalize destination path
        let path = store
            .normalize_secret_path(dest, None, true)
            .map_err(Err::NormalizePath)?;
        let secret = Secret::from(&store, path.to_path_buf());

        let mut plaintext = Plaintext::empty();

        if matcher_add.stdin() {
            plaintext = stdin::read_plaintext(!matcher_main.quiet())?;
        } else if !matcher_add.empty() {
            if let Some(changed) = edit::edit(&plaintext).map_err(Err::Edit)? {
                plaintext = changed;
            }
        }

        // Check if destination already exists if not forcing
        if !matcher_main.force() && path.is_file() {
            eprintln!("A secret at '{}' already exists", path.display(),);
            if !cli::prompt_yes("Overwrite?", Some(true), &matcher_main) {
                if matcher_main.verbose() {
                    eprintln!("Addition cancelled");
                }
                error::quit();
            }
        }

        // Confirm if empty secret should be stored
        if !matcher_main.force() && !matcher_add.empty() && plaintext.is_empty() {
            if !cli::prompt_yes("Secret is empty. Add?", Some(true), &matcher_main) {
                error::quit();
            }
        }

        // Encrypt and write changed plaintext
        // TODO: select proper recipients (use from current file?)
        let recipients = store.recipients()?;
        crypto::context(crypto::PROTO)?
            .encrypt_file(&recipients, plaintext, &path)
            .map_err(Err::Write)?;

        sync.finalize(format!("Add secret to {}", secret.name))?;

        if !matcher_main.quiet() {
            eprintln!("Secret added");
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to normalize destination path")]
    NormalizePath(#[source] anyhow::Error),

    #[error("failed to edit secret in editor")]
    Edit(#[source] anyhow::Error),

    #[error("failed to write changed secret")]
    Write(#[source] anyhow::Error),
}
