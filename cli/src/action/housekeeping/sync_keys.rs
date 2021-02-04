use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::{
    crypto::{self, store::ImportResult},
    Store,
};

use crate::{
    cmd::matcher::{
        housekeeping::{sync_keys::SyncKeysMatcher, HousekeepingMatcher},
        MainMatcher, Matcher,
    },
    util::sync,
};

/// A housekeeping sync-keys action.
pub struct SyncKeys<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> SyncKeys<'a> {
    /// Construct a new sync-keys action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the sync-keys action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_housekeeping = HousekeepingMatcher::with(self.cmd_matches).unwrap();
        let matcher_sync_keys = SyncKeysMatcher::with(self.cmd_matches).unwrap();

        if matcher_main.verbose() {
            eprintln!("Syncing public key files in store with selected recipients...");
        }

        let store = Store::open(matcher_housekeeping.store()).map_err(Err::Store)?;
        let sync = store.sync();

        sync::ensure_ready(&sync);
        sync.prepare()?;

        // Import missing keys into keychain
        if !matcher_sync_keys.no_import() {
            import_missing_keys(&store, matcher_main.quiet(), matcher_main.verbose())
                .map_err(Err::ImportKeys)?;
        }

        // Sync public key files in store
        let recipients = store.recipients().map_err(Err::Load)?;
        crypto::store::store_sync_public_key_files(&store, recipients.keys())?;

        sync.finalize("Sync keys")?;

        if !matcher_main.quiet() {
            eprintln!("Keys synced");
        }

        Ok(())
    }
}

/// Import missing keys from store to keychain.
fn import_missing_keys(store: &Store, quiet: bool, verbose: bool) -> Result<()> {
    if verbose {
        eprintln!("Importing missing public keys from recipients...");
    }

    // Import keys, report results
    for result in crypto::store::import_missing_keys_from_store(&store)? {
        match result {
            ImportResult::Imported(fingerprint) => {
                if !quiet {
                    eprintln!("Imported key to keychain: {}", fingerprint);
                }
            }
            ImportResult::Unavailable(fingerprint) => eprintln!(
                "Cannot import missing public key, not available in store: {}",
                fingerprint
            ),
        }
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to load store recipients")]
    Load(#[source] anyhow::Error),

    #[error("failed to import public keys to keychain")]
    ImportKeys(#[source] anyhow::Error),
}
