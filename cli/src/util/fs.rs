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

/// Calcualte directory size in bytes.
#[cfg(all(feature = "tomb", target_os = "linux"))]
pub fn dir_size(path: &Path) -> Result<u64, Err> {
    fs_extra::dir::get_size(path).map_err(Err::DirSize)
}

#[derive(Debug, Error)]
pub enum Err {
    #[cfg(all(feature = "tomb", target_os = "linux"))]
    #[error("failed to measure directory size")]
    DirSize(#[source] fs_extra::error::Error),
}
