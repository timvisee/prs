use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::{
    crypto::{self, prelude::*, Context},
    Recipients, Secret, Store,
};

#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;
use crate::{
    cmd::matcher::{
        housekeeping::{recrypt::RecryptMatcher, HousekeepingMatcher},
        MainMatcher, Matcher,
    },
    util::{self, error, style, sync},
};

/// A housekeeping recrypt action.
pub struct Recrypt<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Recrypt<'a> {
    /// Construct a new recrypt action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the recrypt action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_housekeeping = HousekeepingMatcher::with(self.cmd_matches).unwrap();
        let matcher_recrypt = RecryptMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_housekeeping.store()).map_err(Err::Store)?;
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        let mut tomb = store.tomb(
            !matcher_main.verbose(),
            matcher_main.verbose(),
            matcher_main.force(),
        );
        let sync = store.sync();

        // Prepare tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::prepare_tomb(&mut tomb, &matcher_main).map_err(Err::Tomb)?;

        // Prepare sync
        sync::ensure_ready(&sync, matcher_recrypt.allow_dirty());
        if !matcher_recrypt.no_sync() {
            sync.prepare()?;
        }

        // Import new keys
        crypto::store::import_missing_keys_from_store(&store).map_err(Err::ImportRecipients)?;

        let secrets = store.secrets(matcher_recrypt.query());

        recrypt(&store, &secrets, &matcher_main)?;

        // Finalize sync
        if !matcher_recrypt.no_sync() {
            sync.finalize("Re-encrypt secrets")?;
        }

        // Finalize tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::finalize_tomb(&mut tomb, &matcher_main, true).map_err(Err::Tomb)?;

        Ok(())
    }
}

/// Re-encrypt all secrets in the given store.
pub fn recrypt_all(store: &Store, matcher_main: &MainMatcher) -> Result<()> {
    recrypt(store, &store.secrets(None), matcher_main)
}

/// Re-encrypt all given secrets.
pub fn recrypt(store: &Store, secrets: &[Secret], matcher_main: &MainMatcher) -> Result<()> {
    let mut context = crate::crypto::context(matcher_main)?;
    let recipients = store.recipients().map_err(Err::Store)?;
    let len = secrets.len();

    let mut failed = Vec::new();

    for (i, secret) in secrets.iter().enumerate() {
        if matcher_main.verbose() {
            eprintln!("[{}/{}] Re-encrypting: {}", i + 1, len, secret.name);
        }

        // Recrypt secret, show status, remember errors
        match recrypt_single(&mut context, secret, &recipients) {
            Ok(_) => {
                if !matcher_main.quiet() {
                    eprintln!("[{}/{}] Re-encrypted: {}", i + 1, len, secret.name);
                }
            }
            Err(err) => {
                eprintln!("[{}/{}] Re-encrypting failed: {}", i + 1, len, secret.name);
                error::print_error(err.context("recrypting failed"));
                failed.push(secret);
            }
        }
    }

    // Show recrypt failures
    if !failed.is_empty() {
        let all = failed.len() >= secrets.len();

        eprintln!();
        error::print_error_msg(format!(
            "Failed to re-encrypt {} of {} secrets",
            failed.len(),
            secrets.len()
        ));

        if !matcher_main.quiet() {
            eprintln!(
                "Use '{}' to try again",
                style::highlight(&format!(
                    "{} housekeeping recrypt{}",
                    util::bin_name(),
                    &if all {
                        " --all".into()
                    } else if failed.len() == 1 {
                        format!(" {}", &failed[0].name)
                    } else {
                        "".into()
                    }
                ))
            );
        }

        error::exit(1);
    }

    Ok(())
}

/// Recrypt a single secret.
fn recrypt_single(context: &mut Context, secret: &Secret, recipients: &Recipients) -> Result<()> {
    let path = &secret.path;
    let plaintext = context.decrypt_file(path).map_err(Err::Read)?;
    context
        .encrypt_file(recipients, plaintext, path)
        .map_err(Err::Write)?;

    Ok(())
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[cfg(all(feature = "tomb", target_os = "linux"))]
    #[error("failed to prepare password store tomb for usage")]
    Tomb(#[source] anyhow::Error),

    #[error("failed to read secret")]
    Read(#[source] anyhow::Error),

    #[error("failed to write changed secret")]
    Write(#[source] anyhow::Error),

    #[error("failed to import store recipients")]
    ImportRecipients(#[source] anyhow::Error),
}
