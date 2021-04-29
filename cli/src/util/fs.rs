use std::path::Path;

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
