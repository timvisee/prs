pub mod crypto;
pub mod store;
pub mod types;

use std::fs;
use std::path::Path;

use anyhow::Result;
use gpgme::Key;
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
                .map(|k| k.fingerprint().unwrap())
                .collect::<Vec<&str>>()
                .join("\n"),
        )
        .map_err(|err| Err::WriteFile(err).into())
    }

    /// Get the list of recipient keys.
    pub fn keys(&self) -> &[Key] {
        &self.keys
    }
}

/// Select all public keys in keyring as recipients.
// TODO: remove this, add better method for obtaining all keyring keys
pub fn all() -> Result<Recipients> {
    Ok(Recipients::from(
        crypto::context()?
            .keys()?
            .into_iter()
            .filter_map(|k| k.ok())
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
