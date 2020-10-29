use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::{
    store::{Secret, Store},
    Recipients,
};

use crate::{
    cmd::matcher::{
        housekeeping::{recrypt::RecryptMatcher, HousekeepingMatcher},
        MainMatcher, Matcher,
    },
    util::sync,
};

/// A housekeeping recrypt action.
pub struct Recrypt<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Recrypt<'a> {
    /// Construct a new recrypt action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the recrypt action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_housekeeping = HousekeepingMatcher::with(self.cmd_matches).unwrap();
        let matcher_recrypt = RecryptMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_housekeeping.store()).map_err(Err::Store)?;
        let sync = store.sync();

        sync::ensure_ready(&sync);
        sync.prepare()?;

        // Import new keys
        Recipients::import_missing_keys_from_store(&store).map_err(Err::ImportRecipients)?;

        let secrets = store.secrets(matcher_recrypt.query());

        recrypt(
            &store,
            &secrets,
            matcher_main.quiet(),
            matcher_main.verbose(),
        )?;

        sync.finalize("Re-encrypt secrets")?;

        Ok(())
    }
}

/// Re-encrypt all secrets in the given store.
pub fn recrypt_all(store: &Store, quiet: bool, verbose: bool) -> Result<()> {
    recrypt(store, &store.secrets(None), quiet, verbose)
}

/// Re-encrypt all given secrets.
pub fn recrypt(store: &Store, secrets: &[Secret], quiet: bool, verbose: bool) -> Result<()> {
    let recipients = store.recipients().map_err(Err::Store)?;
    let len = secrets.len();

    for (i, secret) in secrets.into_iter().enumerate() {
        if verbose {
            eprintln!("[{}/{}] Re-encrypting: {}", i + 1, len, secret.name);
        }

        let path = &secret.path;
        let plaintext = prs_lib::crypto::decrypt_file(path).map_err(Err::Read)?;
        prs_lib::crypto::encrypt_file(&recipients, plaintext, path).map_err(Err::Write)?;

        if !quiet {
            eprintln!("[{}/{}] Re-encrypted: {}", i + 1, len, secret.name);
        }
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to read secret")]
    Read(#[source] anyhow::Error),

    #[error("failed to write changed secret")]
    Write(#[source] anyhow::Error),

    #[error("failed to import store recipients")]
    ImportRecipients(#[source] anyhow::Error),
}
