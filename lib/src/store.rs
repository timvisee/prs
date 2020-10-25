use std::ffi::OsString;
use std::fs;
use std::path::{self, Path, PathBuf};

use anyhow::{ensure, Result};
use thiserror::Error;
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
    pub fn open<P: AsRef<str>>(root: P) -> Result<Self> {
        let root: PathBuf = shellexpand::full(&root)
            .map_err(Err::ExpandPath)?
            .as_ref()
            .into();

        // Make sure store directory exists
        ensure!(root.is_dir(), Err::NoRootDir(root));

        Ok(Self { root })
    }

    /// Get the recipient keys for this store.
    pub fn recipients(&self) -> Result<Recipients> {
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

    /// Normalizes a path for a secret in this store.
    ///
    /// - Ensures path is within store.
    /// - If directory is given, name hint is appended.
    /// - Sets correct extension.
    /// - Creates parent directories if non existant (optional).
    pub fn normalize_secret_path<P: AsRef<Path>>(
        &self,
        target: P,
        name_hint: Option<&str>,
        create_dirs: bool,
    ) -> Result<PathBuf> {
        // Take target as base path
        let mut path = PathBuf::from(target.as_ref());

        // Expand path
        if let Some(path_str) = path.to_str() {
            path = PathBuf::from(
                shellexpand::full(path_str)
                    .map_err(Err::ExpandPath)?
                    .as_ref(),
            );
        }

        let target_is_dir = path.is_dir()
            || target
                .as_ref()
                .to_str()
                .and_then(|s| s.chars().last())
                .map(|s| path::is_separator(s))
                .unwrap_or(false);

        // Strip store prefix
        if let Ok(tmp) = path.strip_prefix(&self.root) {
            path = tmp.into();
        }

        // Make relative
        if path.is_absolute() {
            path = PathBuf::from(format!(".{}{}", path::MAIN_SEPARATOR, path.display()));
        }

        // Prefix store root
        let mut tmp = self.root.clone();
        tmp.push(path);
        path = tmp;

        // Add current secret name if target is dir
        if target_is_dir {
            path.push(name_hint.ok_or_else(|| Err::TargetDirWithoutNamehint(path.clone()))?);
        }

        // Add secret extension if non existent
        let ext: OsString = SECRET_SUFFIX.trim_start_matches(".").into();
        if path.extension() != Some(&ext) {
            let mut tmp = path.as_os_str().to_owned();
            tmp.push(SECRET_SUFFIX);
            path = PathBuf::from(tmp);
        }

        // Create parent dir if it doesn't exist
        if create_dirs {
            let parent = path.parent().unwrap();
            if !parent.is_dir() {
                fs::create_dir_all(parent).map_err(Err::CreateDir)?;
            }
        }

        Ok(path)
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

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to expand store root path")]
    ExpandPath(#[source] shellexpand::LookupError<std::env::VarError>),

    #[error("failed to open password store, not a directory: {0}")]
    NoRootDir(PathBuf),

    #[error("failed to create directory")]
    CreateDir(#[source] std::io::Error),

    #[error("cannot use directory as target without name hint")]
    TargetDirWithoutNamehint(PathBuf),
}
