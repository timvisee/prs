use std::error::Error;
use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

use crate::Recipients;

/// Password store GPG IDs file.
const STORE_GPG_IDS_FILE: &str = ".gpg-id";

/// Password store entry file suffix.
const ENTRY_SUFFIX: &str = ".gpg";

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

    /// Create entry iterator for this store.
    pub fn entry_iter(&self) -> EntryIter {
        EntryIter::new(self.root.clone())
    }

    /// List store password entries.
    pub fn entries(&self) -> Vec<Entry> {
        self.entry_iter().collect()
    }
}

/// A password store entry.
#[derive(Debug, Clone)]
pub struct Entry {
    /// Display name of the entry, relative path to the password store root.
    name: String,

    /// Full path to the password store entry.
    path: PathBuf,
}

impl Entry {
    /// Construct entry at given full path from given store.
    pub fn from(store: &Store, path: PathBuf) -> Self {
        Self::in_root(&store.root, path)
    }

    /// Construct entry at given path in the given password store root.
    pub fn in_root(root: &Path, path: PathBuf) -> Self {
        // TODO: use path.display() as fallback
        let name: String = path
            .strip_prefix(&root)
            .ok()
            .and_then(|f| f.to_str())
            .map(|f| f.trim_end_matches(ENTRY_SUFFIX))
            .unwrap_or_else(|| "?")
            .to_string();
        Self { name, path }
    }

    /// Entry file path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Entry display name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Iterator that walks through password store entries.
///
/// This walks all password store directories, and yields password entries.
/// Hidden files or directories are skipped.
pub struct EntryIter {
    /// Root of the store to walk.
    root: PathBuf,

    /// Directory walker.
    walker: Box<dyn Iterator<Item = DirEntry>>,
}

impl EntryIter {
    /// Create new store entry iterator at given store root.
    pub fn new(root: PathBuf) -> Self {
        let walker = WalkDir::new(&root)
            .into_iter()
            .filter_entry(|e| !is_hidden_subdir(e))
            .filter_map(|e| e.ok())
            .filter(is_entry_file);
        Self {
            root,
            walker: Box::new(walker),
        }
    }
}

impl Iterator for EntryIter {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        self.walker
            .next()
            .map(|e| Entry::in_root(&self.root, e.path().into()))
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

/// Check if given dir entry is a GPG file.
fn is_entry_file(entry: &DirEntry) -> bool {
    entry.file_type().is_file()
        && entry
            .file_name()
            .to_str()
            .map(|s| s.ends_with(ENTRY_SUFFIX))
            .unwrap_or(false)
}
