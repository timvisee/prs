use clap::ArgMatches;

use anyhow::Result;
use prs_lib::{store::Store, types::Plaintext};
use thiserror::Error;

use crate::cmd::matcher::{new::NewMatcher, MainMatcher, Matcher};
use crate::util;

/// New secret action.
pub struct New<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> New<'a> {
    /// Construct a new new action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the new action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_new = NewMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(crate::STORE_DEFAULT_ROOT).map_err(Err::Store)?;

        let dest = matcher_new.destination();

        // Normalize destination path
        let path = store
            .normalize_secret_path(dest, None, true)
            .map_err(Err::NormalizePath)?;

        let plaintext = match util::edit(Plaintext::empty()).map_err(Err::Edit)? {
            Some(changed) => changed,
            None => Plaintext::empty(),
        };

        // Confirm if empty secret should be stored
        if !matcher_main.force() && plaintext.is_empty() {
            if !util::prompt_yes("New secret is empty. Create?", Some(true), &matcher_main) {
                util::quit();
            }
        }

        // Encrypt and write changed plaintext
        // TODO: select proper recipients (use from current file?)
        // TODO: log recipients to encrypt for
        let recipients = store.recipients()?;
        prs_lib::crypto::encrypt_file(&recipients, plaintext, &path).map_err(Err::Write)?;

        if !matcher_main.quiet() {
            println!("Secret created");
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
    Edit(#[source] std::io::Error),

    #[error("failed to write changed secret")]
    Write(#[source] anyhow::Error),
}
