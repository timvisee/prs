use clap::ArgMatches;

use anyhow::Result;
use prs_lib::{store::Store, types::Plaintext};
use thiserror::Error;

use crate::cmd::matcher::{edit::EditMatcher, MainMatcher, Matcher};
use crate::util;

/// A file edit action.
pub struct Edit<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Edit<'a> {
    /// Construct a new edit action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the edit action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_edit = EditMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(crate::STORE_DEFAULT_ROOT).map_err(Err::Store)?;

        let secrets = store.secrets(matcher_edit.query());
        let secret = crate::select_secret(&secrets).ok_or(Err::NoneSelected)?;

        let plaintext = prs_lib::crypto::decrypt_file(&secret.path).map_err(Err::Decrypt)?;

        let plaintext = match edit(plaintext)? {
            Some(changed) => changed,
            None => {
                if !matcher_main.quiet() {
                    eprintln!("Secret is unchanged");
                }
                util::quit();
            }
        };

        // Encrypt and write changed plaintext
        // TODO: select proper recipients (use from current file?)
        // TODO: log recipients to encrypt for
        let recipients = store.recipients()?;
        prs_lib::crypto::encrypt_file(&recipients, plaintext, &secret.path)
            .map_err(Err::Encrypt)?;

        if !matcher_main.quiet() {
            println!("Secret updated");
        }

        Ok(())
    }
}

/// Print the given plaintext to stdout.
// TODO: edit in secure file (/dev/shm) if possible?
fn edit(plaintext: Plaintext) -> Result<Option<Plaintext>> {
    edit::edit_bytes(&plaintext.0)
        .map(|data| {
            Some(data)
                .filter(|data| &plaintext.0 != data)
                .map(Plaintext)
        })
        .map_err(|err| Err::Edit(err).into())
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("no secret selected")]
    NoneSelected,

    #[error("failed to decrypt secret")]
    Decrypt(#[source] anyhow::Error),

    #[error("failed to edit secret in editor")]
    Edit(#[source] std::io::Error),

    #[error("failed to encrypt changed secret")]
    Encrypt(#[source] anyhow::Error),
}
