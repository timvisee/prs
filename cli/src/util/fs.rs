#[cfg(all(feature = "tomb", target_os = "linux"))]
use std::fs;
use std::path::Path;

use anyhow::Result;
use thiserror::Error;

use crate::util::error::{self, ErrorHints};

/// Ensure the given path is a free directory.
///
/// Checks whether the given path is not a directory, or whehter the directory is empty.
/// Quits on error.
pub fn ensure_dir_free(path: &Path) -> Result<(), std::io::Error> {
    // Fine if not a directory
    if !path.is_dir() {
        return Ok(());
    }

    // Fine if no paths in dir
    if path.read_dir()?.count() == 0 {
        return Ok(());
    }

    error::quit_error_msg(
        format!(
            "cannot initialize store, directory already exists: {}",
            path.display(),
        ),
        ErrorHints::default(),
    )
}

/// Check whether the system has SWAP enabled.
#[cfg(all(feature = "tomb", target_os = "linux"))]
pub fn has_swap() -> Result<bool, Err> {
    Ok(fs::read_to_string("/proc/swaps")
        .map_err(Err::HasSwap)?
        .lines()
        .skip(1)
        .next()
        .filter(|l| !l.trim().is_empty())
        .is_some())
}

#[derive(Debug, Error)]
pub enum Err {
    #[cfg(all(feature = "tomb", target_os = "linux"))]
    #[error("failed to check whether system has active SWAP")]
    HasSwap(#[source] std::io::Error),
}
