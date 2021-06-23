//! Password store Tomb functionality.

use std::os::linux::fs::MetadataExt;
use std::path::{Path, PathBuf};

use anyhow::Result;
use thiserror::Error;

use crate::{tomb_bin, Store};

/// Common tomb file suffix.
pub const TOMB_FILE_SUFFIX: &str = ".tomb";

/// Common tomb key file suffix.
pub const TOMB_KEY_FILE_SUFFIX: &str = ".tomb.key";

/// Tomb helper for given store.
pub struct Tomb<'a> {
    /// The store.
    store: &'a Store,
}

impl<'a> Tomb<'a> {
    /// Construct new Tomb helper for given store.
    pub fn new(store: &'a Store) -> Tomb<'a> {
        Self { store }
    }

    /// Find the tomb path.
    ///
    /// Errors if it cannot be found.
    fn find_tomb_path(&self) -> Result<PathBuf> {
        find_tomb_path(&self.store.root).ok_or_else(|| Err::CannotFindTomb.into())
    }

    /// Find the tomb key path.
    ///
    /// Errors if it cannot be found.
    fn find_tomb_key_path(&self) -> Result<PathBuf> {
        find_tomb_key_path(&self.store.root).ok_or_else(|| Err::CannotFindTombKey.into())
    }

    /// Open the tomb.
    pub fn open(&self) -> Result<()> {
        // TODO: ensure tomb isn't already opened
        // TODO: spawn systemd timer to automatically close?

        let tomb = self.find_tomb_path()?;
        let key = self.find_tomb_key_path()?;
        tomb_bin::tomb_open(&tomb, &key, &self.store.root)
    }

    /// Close the tomb.
    pub fn close(&self) -> Result<()> {
        // TODO: ensure tomb is currently open?
        let tomb = self.find_tomb_path()?;
        tomb_bin::tomb_close(&tomb)
    }

    /// Prepare a Tomb store for usage.
    ///
    /// - If this store is a Tomb, the tomb is opened.
    pub fn prepare(&self) -> Result<()> {
        // TODO: return error if dirty?

        // Skip if not a tomb
        if !self.is_tomb() {
            return Ok(());
        }

        // Skip if already open
        if self.is_open()? {
            return Ok(());
        }

        // TODO: do not print if quiet?
        eprintln!("Opening password store Tomb...");

        // TODO: spawn timer to automatically close password store tomb

        // Open tomb
        self.open().map_err(|err| Err::Prepare(err).into())
    }

    /// Finalize the Tomb.
    pub fn finalize(&self) -> Result<()> {
        // This is currently just a placeholder for special closing functionality in the future
        Ok(())
    }

    // /// Initialize tomb.
    // pub fn init(&self) -> Result<()> {
    //     git::git_init(self.path())?;
    //     self.commit_all("Initialize sync with git", true)?;
    //     Ok(())
    // }

    /// Check whether the password store is a tomb.
    ///
    /// This guesses based on existence of some files.
    /// If this returns false you may assume this password store doesn't use a tomb.
    pub fn is_tomb(&self) -> bool {
        find_tomb_path(&self.store.root).is_some()
    }

    /// Check whether the password store is currently opened.
    ///
    /// This guesses based on mount information for the password store directory.
    pub fn is_open(&self) -> Result<bool> {
        // Password store directory must exist
        if !self.store.root.is_dir() {
            return Ok(false);
        }

        // If device ID of store dir and it's parent differ we can assume it is mounted
        if let Some(parent) = self.store.root.parent() {
            let meta_root = self.store.root.metadata().map_err(Err::OpenCheck)?;
            let meta_parent = parent.metadata().map_err(Err::OpenCheck)?;
            return Ok(meta_root.st_dev() != meta_parent.st_dev());
        }

        // TODO: do extensive mount check here

        Ok(false)
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to find tomb file for password store")]
    CannotFindTomb,

    #[error("failed to find tomb key file to unlock password store tomb")]
    CannotFindTombKey,

    #[error("failed to prepare password store tomb for usage")]
    Prepare(#[source] anyhow::Error),

    #[error("failed to check if password store tomb is opened")]
    OpenCheck(#[source] std::io::Error),
}

/// Build list of probable tomb paths for given store root.
fn tomb_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::with_capacity(4);

    // Get parent directory and file name
    let parent = root.parent();
    let file_name = root.file_name().and_then(|n| n.to_str());

    // Same path as store root with .tomb suffix
    if let (Some(parent), Some(file_name)) = (parent, file_name) {
        paths.push(parent.join(format!("{}{}", file_name, TOMB_FILE_SUFFIX)));
    }

    // Path from pass-tomb in store parent and in home
    if let Some(parent) = parent {
        paths.push(parent.join(format!(".password{}", TOMB_FILE_SUFFIX)).into());
    }
    paths.push(format!("~/.password{}", TOMB_FILE_SUFFIX).into());

    paths
}

/// Find tomb path for given store root.
///
/// This does not guarantee that the returned path is an actual tomb file.
/// This is a best effort search.
fn find_tomb_path(root: &Path) -> Option<PathBuf> {
    // TODO: add support for environment variables to set this path
    // TODO: ensure file is large enough to be a tomb (tomb be at least 10 MB)
    tomb_paths(root).into_iter().find(|p| p.is_file())
}

/// Build list of probable tomb key paths for given store root.
fn tomb_key_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::with_capacity(4);

    // Get parent directory and file name
    let parent = root.parent();
    let file_name = root.file_name().and_then(|n| n.to_str());

    // Same path as store root with .tomb suffix
    if let (Some(parent), Some(file_name)) = (parent, file_name) {
        paths.push(parent.join(format!("{}{}", file_name, TOMB_KEY_FILE_SUFFIX)));
    }

    // Path from pass-tomb in store parent and in home
    if let Some(parent) = parent {
        paths.push(
            parent
                .join(format!(".password{}", TOMB_KEY_FILE_SUFFIX))
                .into(),
        );
    }
    paths.push(format!("~/.password{}", TOMB_KEY_FILE_SUFFIX).into());

    paths
}

/// Find tomb key path for given store root.
///
/// This does not guarantee that the returned path is an actual tomb key file.
/// This is a best effort search.
fn find_tomb_key_path(root: &Path) -> Option<PathBuf> {
    // TODO: add support for environment variables to set this path
    tomb_key_paths(root).into_iter().find(|p| p.is_file())
}
