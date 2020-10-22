pub mod crypto;
pub mod types;

use std::error::Error;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use gpgme::Key;

const STORE_GPG_IDS_FILE: &str = ".gpg-id";

/// Represents a password store.
pub struct Store {
    /// Root directory of the password store.
    root: PathBuf,
}

impl Store {
    /// Open a store at the given path.
    pub fn open<P: AsRef<str>>(root: P) -> Self {
        // TODO: make sure path is valid store (exists, contains required files?)

        // Expand path
        // TODO: do full expand, not just tilde
        let root = shellexpand::tilde(&root).as_ref().into();

        Self { root }
    }

    /// Get the recipient keys for this store.
    pub fn recipients(&self) -> Result<Recipients, Box<dyn Error>> {
        // TODO: what to do if ids file does not exist?
        // TODO: what to do if recipients is empty?
        let mut path = self.root.clone();
        path.push(STORE_GPG_IDS_FILE);
        Recipients::find_from_file(path)
    }
}

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
    pub fn find(fingerprints: Vec<String>) -> Result<Recipients, Box<dyn Error>> {
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
    pub fn find_from_file<P: AsRef<Path>>(path: P) -> Result<Recipients, Box<dyn Error>> {
        Self::find(
            fs::read_to_string(path)?
                .lines()
                .filter(|fp| !fp.trim().is_empty())
                .map(|fp| fp.into())
                .collect(),
        )
    }

    /// Get the list of recipient keys.
    pub fn keys(&self) -> &[Key] {
        &self.keys
    }
}
