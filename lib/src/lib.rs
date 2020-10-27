pub mod crypto;
pub mod store;
pub mod types;

use std::fmt;
use std::fs;
use std::path::Path;

use anyhow::Result;
use thiserror::Error;

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
        Self::find(
            fs::read_to_string(path)
                .map_err(Err::ReadFile)?
                .lines()
                .filter(|fp| !fp.trim().is_empty())
                .map(|fp| fp.into())
                .collect(),
        )
    }

    /// Write this list of recipients to a file.
    ///
    /// Overwrites any existing file.
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        fs::write(
            path,
            self.keys
                .iter()
                .map(|k| k.fingerprint(false))
                .collect::<Vec<&str>>()
                .join("\n"),
        )
        .map_err(|err| Err::WriteFile(err).into())
    }

    /// Get the list of recipient keys.
    pub fn keys(&self) -> &[Key] {
        &self.keys
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
}

/// Recipient key.
#[derive(Clone)]
pub struct Key(pub gpgme::Key);

impl Key {
    /// Get fingerprint.
    pub fn fingerprint(&self, short: bool) -> &str {
        let fp = self.0.fingerprint().expect("key does not have fingerprint");
        if short {
            return &fp[fp.len() - 16..];
        }
        fp
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
        write!(f, "{} - {}", self.fingerprint(true), self.user_display())
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

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to read file")]
    ReadFile(#[source] std::io::Error),

    #[error("failed to write file")]
    WriteFile(#[source] std::io::Error),
}
