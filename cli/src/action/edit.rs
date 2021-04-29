use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{
    crypto::{self, prelude::*},
    Store,
};
use thiserror::Error;

use crate::cmd::matcher::{edit::EditMatcher, MainMatcher, Matcher};
use crate::util::{cli, edit, error, secret, select, stdin, sync};

/// Edit secret plaintext action.
pub struct Edit<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Edit<'a> {
    /// Construct a new edit action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the edit action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_edit = EditMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_edit.store()).map_err(Err::Store)?;
        let sync = store.sync();

        sync::ensure_ready(&sync);
        sync.prepare()?;

        let secret =
            select::store_select_secret(&store, matcher_edit.query()).ok_or(Err::NoneSelected)?;

        secret::print_name(matcher_edit.query(), &secret, matcher_main.quiet());

        let mut context = crypto::context(crypto::PROTO)?;
        let mut plaintext = context.decrypt_file(&secret.path).map_err(Err::Read)?;

        if matcher_edit.stdin() {
            plaintext = stdin::read_plaintext(!matcher_main.quiet())?;
        } else {
            plaintext = match edit::edit(&plaintext).map_err(Err::Edit)? {
                Some(changed) => changed,
                None => {
                    if !matcher_main.quiet() {
                        eprintln!("Secret is unchanged");
                    }
                    error::quit();
                }
            };
        }

        // Confirm if empty secret should be stored
        if !matcher_main.force() && plaintext.is_empty() {
            if !cli::prompt_yes("Edited secret is empty. Save?", Some(true), &matcher_main) {
                if matcher_main.verbose() {
                    eprintln!("Secret is unchanged");
                }
                error::quit();
            }
        }

        // Encrypt and write changed plaintext
        // TODO: select proper recipients (use from current file?)
        let recipients = store.recipients()?;
        context
            .encrypt_file(&recipients, plaintext, &secret.path)
            .map_err(Err::Write)?;

        sync.finalize(format!("Edit secret {}", secret.name))?;

        if !matcher_main.quiet() {
            eprintln!("Secret updated");
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

    #[error("failed to read secret")]
    Read(#[source] anyhow::Error),

    #[error("failed to edit secret in editor")]
    Edit(#[source] anyhow::Error),

    #[error("failed to write changed secret")]
    Write(#[source] anyhow::Error),
}
