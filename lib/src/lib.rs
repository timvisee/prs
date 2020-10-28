pub mod crypto;
pub(crate) mod git;
pub mod store;
pub mod sync;
pub mod types;

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use gpgme::Context;
use thiserror::Error;

use crate::store::Store;

/// Password store GPG IDs file.
const STORE_GPG_IDS_FILE: &str = ".gpg-id";

/// Password store public key directory.
const STORE_PUB_KEY_DIR: &str = ".public-keys/";

/// List of recipient keys.
pub struct Recipients {
    keys: Vec<Key>,
}

impl Recipients {
    /// Construct recipients list from given keys.
    pub fn from(keys: Vec<Key>) -> Self {
        Self { keys }
    }

    /// Find recipients based on given fingerprint list.
    pub fn find(fingerprints: Vec<String>) -> Result<Recipients> {
        let keys = if fingerprints.is_empty() {
            vec![]
        } else {
            crypto::context()?
                .find_keys(fingerprints)?
                .filter_map(|x| x.ok())
                .filter(|k| k.can_encrypt())
                .map(|k| k.into())
                .collect()
        };
        Ok(Recipients::from(keys))
    }

    /// Find recipients based on fingerprints listed in given file.
    pub fn find_from_file<P: AsRef<Path>>(path: P) -> Result<Recipients> {
        Self::find(read_fingerprints_file(path)?)
    }

    /// Get the list of recipient keys.
    pub fn keys(&self) -> &[Key] {
        &self.keys
    }

    /// Check whether this recipient list has the given fingerprint.
    fn has_fingerprint(&self, fingerprint: &str) -> bool {
        let fp = fingerprint.trim().to_uppercase();
        self.keys.iter().any(|k| k.fingerprint(false) == fp)
    }

    /// Add the given key.
    ///
    /// Does not add if already existant.
    pub fn add(&mut self, key: Key) {
        if !self.keys.contains(&key) {
            self.keys.push(key);
        }
    }

    /// Remove the given key if existant.
    pub fn remove(&mut self, key: &Key) {
        self.keys.retain(|k| k != key);
    }

    /// Remove the given keys.
    ///
    /// Keys that are not found are ignored.
    pub fn remove_many(&mut self, keys: &[Key]) {
        self.keys.retain(|k| !keys.contains(k));
    }

    /// Load recipients from a store.
    pub fn load(store: &Store) -> Result<Self> {
        // TODO: what to do if ids file does not exist?
        // TODO: what to do if recipients is empty?
        // TODO: what to do if key listed in file is not found, attempt to install?
        Recipients::find_from_file(store_gpg_ids_file(&store))
    }

    /// Save this list of recipients to the store.
    ///
    /// This overwrites any existing recipient list.
    pub fn save(&self, store: &Store) -> Result<()> {
        self.write_to_file(store_gpg_ids_file(store))?;
        self.sync_public_key_files(store)
        // TODO: import missing keys to system?
    }

    /// Sync public key files in store with selected recipients.
    ///
    /// - Removes obsolete keys that are not a selected recipient
    /// - Adds missing keys that are a recipient
    pub fn sync_public_key_files(&self, store: &Store) -> Result<()> {
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
                    .map(|fp| (e.path(), format_fingerprint(fp)))
            })
            .collect();

        // Remove unused keys
        for (path, _) in files.iter().filter(|(_, fp)| !self.has_fingerprint(fp)) {
            fs::remove_file(path).map_err(Err::SyncKeyFiles)?;
        }

        // Add missing keys
        let mut context: Option<_> = None;
        for (key, fp) in self
            .keys
            .iter()
            .map(|k| (k, k.fingerprint(false)))
            .filter(|(_, fp)| !files.iter().any(|(_, other)| fp == other))
        {
            // Lazy load context
            if context.is_none() {
                context = Some(crypto::context()?);
            }

            // Export public key
            let data = export_key(context.as_mut().unwrap(), key).map_err(Err::Export)?;

            // Write public key to disk
            let path = dir.join(&fp);
            fs::write(path, data).map_err(Err::SyncKeyFiles)?;
        }

        Ok(())
    }

    /// Write recipient fingerprints to file.
    ///
    /// Overwrites any existing file.
    fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        fs::write(
            path,
            self.keys
                .iter()
                .map(|k| k.fingerprint(false))
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .map_err(|err| Err::WriteFile(err).into())
    }
}

/// Read GPG fingerprints from the given file.
pub fn read_fingerprints_file<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    Ok(fs::read_to_string(path)
        .map_err(Err::ReadFile)?
        .lines()
        .filter(|fp| !fp.trim().is_empty())
        .map(|fp| fp.into())
        .collect())
}

/// Filter list of fingerprints.
///
/// Keep list of unimported fingerprints.
pub fn filter_imported_fingerprints(fingerprints: Vec<String>) -> Result<Vec<String>> {
    let mut context = crypto::context()?;
    Ok(fingerprints
        .into_iter()
        .filter(|fp| context.get_key(fp).is_err())
        .collect())
}

/// Get the GPG IDs file for a store.
pub fn store_gpg_ids_file(store: &Store) -> PathBuf {
    store.root.join(STORE_GPG_IDS_FILE)
}

/// Get the public keys directory for a store.
pub fn store_public_keys_dir(store: &Store) -> PathBuf {
    store.root.join(STORE_PUB_KEY_DIR)
}

/// Export the given key as bytes.
pub fn export_key(context: &mut Context, key: &Key) -> Result<Vec<u8>, gpgme::Error> {
    // Export public key
    let mut data: Vec<u8> = vec![];
    context.export_keys(&[key.0.clone()], gpgme::ExportMode::empty(), &mut data)?;

    // Assert we're exporting a public key
    let data_str = std::str::from_utf8(&data).expect("exported key is invalid UTF-8");
    assert!(
        !data_str.contains("PRIVATE"),
        "exported key contains PRIVATE, blocked to prevent accidentally leaking secret key"
    );
    assert!(
        data_str.contains("PUBLIC"),
        "exported key must contain PUBLIC, blocked to prevent accidentally leaking secret key"
    );
    Ok(data)
}

/// Recipient key.
#[derive(Clone)]
pub struct Key(pub gpgme::Key);

impl Key {
    /// Get fingerprint.
    pub fn fingerprint(&self, short: bool) -> String {
        let fp = self.0.fingerprint().expect("key does not have fingerprint");
        if short {
            return format_fingerprint(&fp[fp.len() - 16..]);
        }
        format_fingerprint(fp)
    }

    /// Format user data to displayable string.
    pub fn user_display(&self) -> String {
        self.0
            .user_ids()
            .map(|user| {
                let mut parts = vec![];
                if let Ok(name) = user.name() {
                    if !name.trim().is_empty() {
                        parts.push(name.into());
                    }
                }
                if let Ok(comment) = user.comment() {
                    if !comment.trim().is_empty() {
                        parts.push(format!("({})", comment));
                    }
                }
                if let Ok(email) = user.email() {
                    if !email.trim().is_empty() {
                        parts.push(format!("<{}>", email));
                    }
                }
                parts.join(" ")
            })
            .collect::<Vec<_>>()
            .join("; ")
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.0.id_raw() == other.0.id_raw() && self.0.fingerprint_raw() == other.0.fingerprint_raw()
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[GPG] {} - {}",
            self.fingerprint(true),
            self.user_display()
        )
    }
}

impl From<gpgme::Key> for Key {
    fn from(key: gpgme::Key) -> Self {
        Self(key)
    }
}

/// Select all public keys from keychain usable as recipient.
// TODO: does this include private keys for encrypting?
// TODO: remove this, add better method for obtaining all keyring keys
pub fn all() -> Result<Recipients> {
    Ok(Recipients::from(
        crypto::context()?
            .keys()?
            .into_iter()
            .filter_map(|k| k.ok())
            .filter(|k| k.can_encrypt())
            .map(|k| k.into())
            .collect(),
    ))
}

/// Reformat the given fingerprint.
fn format_fingerprint<S: AsRef<str>>(fingerprint: S) -> String {
    fingerprint.as_ref().trim().to_uppercase()
}

#[derive(Debug, Error)]
pub enum Err {
    // TODO: add load/save erros
    #[error("failed to read file")]
    ReadFile(#[source] std::io::Error),

    #[error("failed to write file")]
    WriteFile(#[source] std::io::Error),

    #[error("failed to sync public key files")]
    SyncKeyFiles(#[source] std::io::Error),

    #[error("failed to export key")]
    Export(#[source] gpgme::Error),
}
