use std::error::Error;
use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

use crate::Recipients;

/// Password store GPG IDs file.
const STORE_GPG_IDS_FILE: &str = ".gpg-id";

/// Password store secret file suffix.
pub const SECRET_SUFFIX: &str = ".gpg";

/// Represents a password store.
pub struct Store {
    /// Root directory of the password store.
    ///
    /// This path is always absolute.
    pub root: PathBuf,
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

    /// Create secret iterator for this store.
    pub fn secret_iter(&self) -> SecretIter {
        SecretIter::new(self.root.clone())
    }

    /// List store password secrets.
    pub fn secrets(&self, filter: Option<String>) -> Vec<Secret> {
        self.secret_iter().filter(filter).collect()
    }
}

/// A password store secret.
#[derive(Debug, Clone)]
pub struct Secret {
    /// Display name of the secret, relative path to the password store root.
    pub name: String,

    /// Full path to the password store secret.
    pub path: PathBuf,
}

impl Secret {
    /// Construct secret at given full path from given store.
    pub fn from(store: &Store, path: PathBuf) -> Self {
        Self::in_root(&store.root, path)
    }

    /// Construct secret at given path in the given password store root.
    pub fn in_root(root: &Path, path: PathBuf) -> Self {
        // TODO: use path.display() as fallback
        let name: String = path
            .strip_prefix(&root)
            .ok()
            .and_then(|f| f.to_str())
            .map(|f| f.trim_end_matches(SECRET_SUFFIX))
            .unwrap_or_else(|| "?")
            .to_string();
        Self { name, path }
    }
}

/// Iterator that walks through password store secrets.
///
/// This walks all password store directories, and yields password secrets.
/// Hidden files or directories are skipped.
pub struct SecretIter {
    /// Root of the store to walk.
    root: PathBuf,

    /// Directory walker.
    walker: Box<dyn Iterator<Item = DirEntry>>,
}

impl SecretIter {
    /// Create new store secret iterator at given store root.
    pub fn new(root: PathBuf) -> Self {
        let walker = WalkDir::new(&root)
            .into_iter()
            .filter_entry(|e| !is_hidden_subdir(e))
            .filter_map(|e| e.ok())
            .filter(is_secret_file);
        Self {
            root,
            walker: Box::new(walker),
        }
    }

    /// Transform into a filtered secret iterator.
    pub fn filter(self, filter: Option<String>) -> FilterSecretIter<Self> {
        FilterSecretIter::new(self, filter)
    }
}

impl Iterator for SecretIter {
    type Item = Secret;

    fn next(&mut self) -> Option<Self::Item> {
        self.walker
            .next()
            .map(|e| Secret::in_root(&self.root, e.path().into()))
    }
}

/// Check if given WalkDir DirEntry is hidden sub-directory.
fn is_hidden_subdir(entry: &DirEntry) -> bool {
    entry.depth() > 0
        && entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with("."))
            .unwrap_or(false)
}

/// Check if given WalkDir DirEntry is a secret file.
fn is_secret_file(entry: &DirEntry) -> bool {
    entry.file_type().is_file()
        && entry
            .file_name()
            .to_str()
            .map(|s| s.ends_with(SECRET_SUFFIX))
            .unwrap_or(false)
}

/// Iterator that wraps a `SecretIter` with a filter.
pub struct FilterSecretIter<I>
where
    I: Iterator<Item = Secret>,
{
    inner: I,
    filter: Option<String>,
}

impl<I> FilterSecretIter<I>
where
    I: Iterator<Item = Secret>,
{
    /// Construct a new filter secret iterator.
    pub fn new(inner: I, filter: Option<String>) -> Self {
        Self { inner, filter }
    }
}

impl<I> Iterator for FilterSecretIter<I>
where
    I: Iterator<Item = Secret>,
{
    type Item = Secret;

    fn next(&mut self) -> Option<Self::Item> {
        if self.filter.is_none() {
            return self.inner.next();
        }

        let filter = self.filter.as_ref().unwrap();
        while let Some(secret) = self.inner.next() {
            if secret.name.contains(filter) {
                return Some(secret);
            }
        }

        None
    }
}
