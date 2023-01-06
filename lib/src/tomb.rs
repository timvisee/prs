//! Password store Tomb functionality.

use std::env;
use std::os::linux::fs::MetadataExt;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use thiserror::Error;

use crate::crypto::Proto;
pub use crate::tomb_bin::TombSettings;
use crate::util;
use crate::{systemd_bin, tomb_bin, Key, Store};

/// Default time after which to automatically close the password tomb.
pub const TOMB_AUTO_CLOSE_SEC: u32 = 5 * 60;

/// Common tomb file suffix.
pub const TOMB_FILE_SUFFIX: &str = ".tomb";

/// Common tomb key file suffix.
pub const TOMB_KEY_FILE_SUFFIX: &str = ".tomb.key";

/// Name of SSH client process.
pub const SSH_PROCESS_NAME: &str = "ssh";

/// Tomb helper for given store.
pub struct Tomb<'a> {
    /// The store.
    store: &'a Store,

    /// Tomb settings.
    pub settings: TombSettings,
}

impl<'a> Tomb<'a> {
    /// Construct new Tomb helper for given store.
    pub fn new(store: &'a Store, quiet: bool, verbose: bool, force: bool) -> Tomb<'a> {
        Self {
            store,
            settings: TombSettings {
                quiet,
                verbose,
                force,
            },
        }
    }

    /// Find the tomb path.
    ///
    /// Errors if it cannot be found.
    pub fn find_tomb_path(&self) -> Result<PathBuf> {
        find_tomb_path(&self.store.root).ok_or_else(|| Err::CannotFindTomb.into())
    }

    /// Find the tomb key path.
    ///
    /// Errors if it cannot be found.
    pub fn find_tomb_key_path(&self) -> Result<PathBuf> {
        find_tomb_key_path(&self.store.root).ok_or_else(|| Err::CannotFindTombKey.into())
    }

    /// Open the tomb.
    ///
    /// This will keep the tomb open until it is manually closed. See `start_timer()`.
    ///
    /// On success this may return a list with soft-fail errors.
    pub fn open(&self) -> Result<Vec<Err>> {
        // Open tomb
        let tomb = self.find_tomb_path()?;
        let key = self.find_tomb_key_path()?;
        tomb_bin::tomb_open(&tomb, &key, &self.store.root, None, self.settings)
            .map_err(Err::Open)?;

        // Soft fail on following errors, collect them
        let mut errs = vec![];

        // Change mountpoint directory permissions to current user
        if let Err(err) =
            util::fs::sudo_chown_current_user(&self.store.root, false).map_err(Err::Chown)
        {
            errs.push(err);
        }

        Ok(errs)
    }

    /// Resize the tomb.
    ///
    /// The Tomb must not be mounted and the size must be larger than the current.
    pub fn resize(&self, mbs: u32) -> Result<()> {
        let tomb = self.find_tomb_path()?;
        let key = self.find_tomb_key_path()?;
        tomb_bin::tomb_resize(&tomb, &key, mbs, self.settings).map_err(Err::Resize)?;
        Ok(())
    }

    /// Close the tomb.
    pub fn close(&self) -> Result<()> {
        let tomb = self.find_tomb_path()?;

        // Kill SSH clients that still have a persistent session open for this store
        util::git::kill_ssh_by_session(self.store);

        tomb_bin::tomb_close(&tomb, self.settings)
    }

    /// Slam all open tombs.
    ///
    /// Warning: this may be dangerous and could have unwanted side effects. This also closes
    /// non-password Tombs and kills all programs using it.
    pub fn slam(&self) -> Result<()> {
        tomb_bin::tomb_slam(self.settings)
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

        if !self.settings.quiet {
            eprintln!("Opening password store Tomb...");
        }

        // Open tomb, set up auto close timer
        self.open().map_err(Err::Prepare)?;
        self.start_timer(TOMB_AUTO_CLOSE_SEC, false)
            .map_err(Err::Prepare)?;

        eprintln!();
        if self.settings.verbose {
            eprintln!("Opened password store, automatically closing in 5 seconds");
        }

        Ok(())
    }

    /// Set up a timer to automatically close password store tomb.
    ///
    /// TODO: add support for non-systemd systems
    pub fn start_timer(&self, sec: u32, force: bool) -> Result<()> {
        // Figure out tomb path and name
        let tomb_path = self.find_tomb_path()?;
        let name = tomb_bin::name(&tomb_path).unwrap_or(".unwrap");
        let unit = format!("prs-tomb-close@{}.service", name);

        // Skip if already running
        if !force && systemd_bin::systemd_has_timer(&unit).map_err(Err::AutoCloseTimer)? {
            return Ok(());
        }

        // Spawn timer to automatically close tomb
        // TODO: better method to find current exe path
        // TODO: do not hardcode exe, command and store path
        systemd_bin::systemd_cmd_timer(
            sec,
            "prs tomb close timer",
            &unit,
            &[
                std::env::current_exe()
                    .expect("failed to determine current exe")
                    .to_str()
                    .expect("current exe contains invalid UTF-8"),
                "tomb",
                "--store",
                self.store
                    .root
                    .to_str()
                    .expect("password store path contains invalid UTF-8"),
                "close",
                "--try",
                "--verbose",
            ],
        )
        .map_err(Err::AutoCloseTimer)?;

        Ok(())
    }

    /// Check whether the timer is running.
    pub fn has_timer(&self) -> Result<bool> {
        // Figure out tomb path and name
        let tomb_path = self.find_tomb_path()?;
        let name = tomb_bin::name(&tomb_path).unwrap_or(".unwrap");
        let unit = format!("prs-tomb-close@{}.service", name);

        systemd_bin::systemd_has_timer(&unit).map_err(|err| Err::AutoCloseTimer(err).into())
    }

    /// Stop automatic close timer if any is running.
    pub fn stop_timer(&self) -> Result<()> {
        // Figure out tomb path and name
        let tomb_path = self.find_tomb_path()?;
        let name = tomb_bin::name(&tomb_path).unwrap_or(".unwrap");
        let unit = format!("prs-tomb-close@{}.service", name);

        // We're done if none is running
        if !systemd_bin::systemd_has_timer(&unit).map_err(Err::AutoCloseTimer)? {
            return Ok(());
        }

        systemd_bin::systemd_remove_timer(&unit).map_err(Err::AutoCloseTimer)?;
        Ok(())
    }

    /// Finalize the Tomb.
    pub fn finalize(&self) -> Result<()> {
        // This is currently just a placeholder for special closing functionality in the future
        Ok(())
    }

    /// Initialize tomb.
    ///
    /// `mbs` is the size in megabytes.
    ///
    /// The given GPG key is used to encrypt the Tomb key with.
    ///
    /// # Panics
    ///
    /// Panics if given key is not a GPG key.
    pub fn init(&self, key: &Key, mbs: u32) -> Result<()> {
        // Assert key is GPG
        assert_eq!(key.proto(), Proto::Gpg, "key for Tomb is not a GPG key");

        // TODO: map errors

        // TODO: we need these paths even though tomb does not exist yet
        let tomb_file = tomb_paths(&self.store.root).first().unwrap().to_owned();
        let key_file = tomb_key_paths(&self.store.root).first().unwrap().to_owned();
        let store_tmp_dir =
            util::fs::append_file_name(&self.store.root, ".tomb-init").map_err(Err::Init)?;

        // Dig tomb, forge key, lock tomb with key, open tomb
        tomb_bin::tomb_dig(&tomb_file, mbs, self.settings).map_err(Err::Init)?;
        tomb_bin::tomb_forge(&key_file, key, self.settings).map_err(Err::Init)?;
        tomb_bin::tomb_lock(&tomb_file, &key_file, key, self.settings).map_err(Err::Init)?;
        tomb_bin::tomb_open(
            &tomb_file,
            &key_file,
            &store_tmp_dir,
            Some(key),
            self.settings,
        )
        .map_err(Err::Init)?;

        // Change temporary mountpoint directory permissions to current user
        util::fs::sudo_chown_current_user(&store_tmp_dir, true).map_err(Err::Chown)?;

        // Copy password store contents
        util::fs::copy_dir_contents(&self.store.root, &store_tmp_dir).map_err(Err::Init)?;

        // Close tomb
        tomb_bin::tomb_close(&tomb_file, self.settings).map_err(Err::Init)?;
        util::fs::sudo_chown_current_user(&store_tmp_dir, true).map_err(Err::Chown)?;

        // Remove both main and temporary store
        fs_extra::dir::remove(&self.store.root).map_err(|err| Err::Init(anyhow!(err)))?;
        fs_extra::dir::remove(&store_tmp_dir).map_err(|err| Err::Init(anyhow!(err)))?;

        // Open tomb as regular
        // TODO: do something with Ok(errors)?
        self.open()?;

        Ok(())
    }

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

    /// Fetch Tomb size statistics.
    ///
    /// This attempts to gather password store and tomb size statistics, whether this store is a
    /// tomb or not.
    ///
    /// This is expensive.
    pub fn fetch_size_stats(&self) -> Result<TombSize> {
        // Get sizes depending on whether this store uses a tomb
        match self.find_tomb_path() {
            Ok(tomb_path) => {
                let store = if self.is_open().unwrap_or(false) {
                    util::fs::dir_size(&self.store.root).ok()
                } else {
                    None
                };
                let tomb_file = tomb_path.metadata().map(|m| m.len()).ok();

                Ok(TombSize { store, tomb_file })
            }
            Err(_) => Ok(TombSize {
                store: util::fs::dir_size(&self.store.root).ok(),
                tomb_file: None,
            }),
        }
    }
}

/// Slam all open tombs.
///
/// Warning: this may be dangerous and could have unwanted side effects. This also closes
/// non-password Tombs and kills all programs using it.
pub fn slam(settings: TombSettings) -> Result<()> {
    tomb_bin::tomb_slam(settings).map_err(Err::Slam)?;
    Ok(())
}

/// Holds information for password store Tomb sizes.
#[derive(Debug, Copy, Clone)]
pub struct TombSize {
    /// Store directory.
    pub store: Option<u64>,

    /// Tomb file size.
    pub tomb_file: Option<u64>,
}

impl TombSize {
    /// Get Tomb file size in MBs.
    pub fn tomb_file_size_mbs(&self) -> Option<u32> {
        self.tomb_file.map(|s| (s / 1024 / 1024) as u32)
    }

    /// Get the desired Tomb size in megabytes based on the current state.
    ///
    /// Currently twice the password store size, defaults to minimum of 10.
    pub fn desired_tomb_size(&self) -> u32 {
        self.store
            .map(|bytes| ((bytes * 3) / 1024 / 1024).max(10) as u32)
            .unwrap_or(10)
    }

    /// Determine whether the password store should be resized.
    pub fn should_resize(&self) -> bool {
        // TODO: determine this based on 'tomb list' output
        self.store
            .zip(self.tomb_file)
            .map(|(store, tomb_file)| store * 2 > tomb_file)
            .unwrap_or(false)
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

    #[error("failed to initialize new password store tomb")]
    Init(#[source] anyhow::Error),

    #[error("failed to open password store tomb through tomb CLI")]
    Open(#[source] anyhow::Error),

    #[error("failed to resize password store tomb through tomb CLI")]
    Resize(#[source] anyhow::Error),

    #[error("failed to slam all open tombs through tomb CLI")]
    Slam(#[source] anyhow::Error),

    #[error("failed to change permissions to current user for tomb mountpoint")]
    Chown(#[source] anyhow::Error),

    #[error("failed to check if password store tomb is opened")]
    OpenCheck(#[source] std::io::Error),

    #[error("failed to set up systemd timer to auto close password store tomb")]
    AutoCloseTimer(#[source] anyhow::Error),
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
        paths.push(parent.join(format!(".password{}", TOMB_FILE_SUFFIX)));
    }
    paths.push(format!("~/.password{}", TOMB_FILE_SUFFIX).into());

    paths
}

/// Find tomb path for given store root.
///
/// Uses `PASSWORD_STORE_TOMB_FILE` if set.
/// This does not guarantee that the returned path is an actual tomb file.
/// This is a best effort search.
fn find_tomb_path(root: &Path) -> Option<PathBuf> {
    // Take path from environment variable
    if let Ok(path) = env::var("PASSWORD_STORE_TOMB_FILE") {
        return Some(path.into());
    }

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
        paths.push(parent.join(format!(".password{}", TOMB_KEY_FILE_SUFFIX)));
    }
    paths.push(format!("~/.password{}", TOMB_KEY_FILE_SUFFIX).into());

    paths
}

/// Find tomb key path for given store root.
///
/// Uses `PASSWORD_STORE_TOMB_KEY` if set.
/// This does not guarantee that the returned path is an actual tomb key file.
/// This is a best effort search.
fn find_tomb_key_path(root: &Path) -> Option<PathBuf> {
    // Take path from environment variable
    if let Ok(path) = env::var("PASSWORD_STORE_TOMB_KEY") {
        return Some(path.into());
    }

    tomb_key_paths(root).into_iter().find(|p| p.is_file())
}
