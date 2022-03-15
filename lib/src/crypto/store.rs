//! Helpers to use recipients with password store.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use thiserror::Error;

use super::{prelude::*, recipients::Recipients, util, Config, ContextPool, Key, Proto};
use crate::Store;

/// Password store GPG IDs file.
const STORE_GPG_IDS_FILE: &str = ".gpg-id";

/// Password store public key directory.
const STORE_PUB_KEY_DIR: &str = ".public-keys/";

/// Get the GPG IDs file for a store.
pub fn store_gpg_ids_file(store: &Store) -> PathBuf {
    store.root.join(STORE_GPG_IDS_FILE)
}

/// Get the public keys directory for a store.
pub fn store_public_keys_dir(store: &Store) -> PathBuf {
    store.root.join(STORE_PUB_KEY_DIR)
}

/// Read GPG fingerprints from store.
pub fn store_read_gpg_fingerprints(store: &Store) -> Result<Vec<String>> {
    let path = store_gpg_ids_file(store);
    if path.is_file() {
        read_fingerprints(path)
    } else {
        Ok(vec![])
    }
}

/// Write GPG fingerprints to a store.
///
/// Overwrites any existing file.
pub fn store_write_gpg_fingerprints<S: AsRef<str>>(
    store: &Store,
    fingerprints: &[S],
) -> Result<()> {
    write_fingerprints(store_gpg_ids_file(store), fingerprints)
}

/// Read fingerprints from the given file.
fn read_fingerprints<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    Ok(fs::read_to_string(path)
        .map_err(Err::ReadFile)?
        .lines()
        .filter(|fp| !fp.trim().is_empty())
        .map(|fp| fp.into())
        .collect())
}

/// Write fingerprints to the given file.
fn write_fingerprints<P: AsRef<Path>, S: AsRef<str>>(path: P, fingerprints: &[S]) -> Result<()> {
    fs::write(
        path,
        fingerprints
            .iter()
            .map(|k| k.as_ref())
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .map_err(|err| Err::WriteFile(err).into())
}

/// Load the keys for the given store.
///
/// This will try to load the keys for all configured protocols, and errors if it fails.
pub fn store_load_keys(store: &Store) -> Result<Vec<Key>> {
    let mut keys = Vec::new();

    // TODO: what to do if ids file does not exist?
    // TODO: what to do if recipients is empty?
    // TODO: what to do if key listed in file is not found, attempt to install?

    // Load GPG keys
    // TODO: do not crash here if GPG ids file is not found!
    let fingerprints = store_read_gpg_fingerprints(store)?;

    if !fingerprints.is_empty() {
        let mut context = super::context(&crate::CONFIG)?;
        let fingerprints: Vec<_> = fingerprints.iter().map(|fp| fp.as_str()).collect();
        keys.extend(context.find_public_keys(&fingerprints)?);
    }

    // NEWPROTO: if a new proto is added, keys for a store should be loaded here

    Ok(keys)
}

/// Load the recipients for the given store.
///
/// This will try to load the recipient keys for all configured protocols, and errors if it fails.
pub fn store_load_recipients(store: &Store) -> Result<Recipients> {
    Ok(Recipients::from(store_load_keys(store)?))
}

/// Save the keys for the given store.
///
/// This overwrites any existing recipient keys.
pub fn store_save_keys(store: &Store, keys: &[Key]) -> Result<()> {
    // Save GPG keys
    let gpg_fingerprints: Vec<_> = keys
        .iter()
        .filter(|key| key.proto() == Proto::Gpg)
        .map(|key| key.fingerprint(false))
        .collect();
    store_write_gpg_fingerprints(store, &gpg_fingerprints)?;

    // Sync public keys for all proto's
    store_sync_public_key_files(store, keys)?;

    // TODO: import missing keys to system?

    Ok(())
}

/// Save the keys for the given store.
///
/// This overwrites any existing recipient keys.
pub fn store_save_recipients(store: &Store, recipients: &Recipients) -> Result<()> {
    store_save_keys(store, recipients.keys())
}

/// Sync public key files in store with selected recipients.
///
/// - Removes obsolete keys that are not a selected recipient
/// - Adds missing keys that are a recipient
///
/// This syncs public key files for all protocols. This is because the public key files themselves
/// don't specify what protocol they use. All public key files and keys must therefore be taken
/// into consideration all at once.
pub fn store_sync_public_key_files(store: &Store, keys: &[Key]) -> Result<()> {
    // Get public keys directory, ensure it exists
    let dir = store_public_keys_dir(store);
    fs::create_dir_all(&dir).map_err(Err::SyncKeyFiles)?;

    // List key files in keys directory
    let files: Vec<(PathBuf, String)> = dir
        .read_dir()
        .map_err(Err::SyncKeyFiles)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|f| f.is_file()).unwrap_or(false))
        .filter_map(|e| {
            e.file_name()
                .to_str()
                .map(|fp| (e.path(), util::format_fingerprint(fp)))
        })
        .collect();

    // Remove unused keys
    for (path, _) in files
        .iter()
        .filter(|(_, fp)| !util::keys_contain_fingerprint(keys, fp))
    {
        fs::remove_file(path).map_err(Err::SyncKeyFiles)?;
    }

    // Add missing keys
    let mut contexts = ContextPool::empty();
    for (key, fp) in keys
        .iter()
        .map(|k| (k, k.fingerprint(false)))
        .filter(|(_, fp)| !files.iter().any(|(_, other)| fp == other))
    {
        // Lazy load compatible context
        let proto = key.proto();
        let config = Config::from(proto);
        let context = contexts.get_mut(&config)?;

        // Export public key to disk
        let path = dir.join(&fp);
        context.export_key_file(key.clone(), &path)?;
    }

    // NEWPROTO: if a new proto is added, public keys should be synced here

    Ok(())
}

/// Import keys from store that are missing in the keychain.
pub fn import_missing_keys_from_store(store: &Store) -> Result<Vec<ImportResult>> {
    // Get public keys directory, ensure it exists
    let dir = store_public_keys_dir(store);
    if !dir.is_dir() {
        return Ok(vec![]);
    }

    // Cache protocol contexts
    let mut contexts = ContextPool::empty();
    let mut results = Vec::new();

    // Check for missing GPG keys based on fingerprint, import them
    let gpg_fingerprints = store_read_gpg_fingerprints(store)?;
    for fingerprint in gpg_fingerprints {
        let context = contexts.get_mut(&crate::CONFIG)?;
        if context.get_public_key(&fingerprint).is_err() {
            let path = &store_public_keys_dir(store).join(&fingerprint);
            if path.is_file() {
                context.import_key_file(path)?;
                results.push(ImportResult::Imported(fingerprint));
            } else {
                results.push(ImportResult::Unavailable(fingerprint));
            }
        }
    }

    // NEWPROTO: if a new proto is added, import missing keys here

    Ok(results)
}

/// Missing key import results.
pub enum ImportResult {
    /// Key with given fingerprint was imported into keychain.
    Imported(String),

    /// Key with given fingerprint was not found and was not imported in keychain.
    Unavailable(String),
}

/// Recipients extension for store functionality.
pub trait StoreRecipients {
    /// Load recipients from given store.
    fn load(store: &Store) -> Result<Recipients>;

    /// Save recipients to given store.
    fn save(&self, store: &Store) -> Result<()>;
}

impl StoreRecipients for Recipients {
    /// Load recipients from given store.
    fn load(store: &Store) -> Result<Recipients> {
        store_load_recipients(store)
    }

    /// Save recipients to given store.
    fn save(&self, store: &Store) -> Result<()> {
        store_save_recipients(store, self)
    }
}

/// Store crypto error.
#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to write to file")]
    WriteFile(#[source] std::io::Error),

    #[error("failed to read from file")]
    ReadFile(#[source] std::io::Error),

    #[error("failed to sync public key files")]
    SyncKeyFiles(#[source] std::io::Error),
}
