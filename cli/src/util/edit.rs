use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use thiserror::Error;

use prs_lib::Plaintext;

/// Shared memory tmpfs mount path on Unix (Linux?) systems.
#[cfg(unix)]
const UNIX_DIR_SHM: &str = "/dev/shm";

/// Edit given plaintext in default editor.
///
/// Only returns `Plaintext` if changed.
pub fn edit(plaintext: &Plaintext) -> Result<Option<Plaintext>> {
    // Create secure temporary file for secret
    let mut builder = edit::Builder::new();
    builder.prefix(".prs-secret-");
    builder.suffix(".txt");
    let file = builder.tempfile_in(dir()).map_err(Err::Create)?;

    // Show Windows users where to save the file because notepad doesn't remember properly
    #[cfg(windows)]
    eprintln!(
        "Opening editor, save edited file at: {}",
        file.path().display()
    );

    // Attempt to edit plaintext, explicitly close/remove file, handle errors last
    let new_plaintext = write_edit_read(plaintext, file.path());
    file.close().map_err(Err::Close)?;
    let new_plaintext = new_plaintext?;

    // Return none if unchanged
    if plaintext == &new_plaintext {
        return Ok(None);
    }

    Ok(Some(new_plaintext))
}

/// Edit the given plaintext in the given file.
///
/// This writes the plaintext to the file, opens it in the default editor, and reads it after
/// closing.
fn write_edit_read(plaintext: &Plaintext, file: &Path) -> Result<Plaintext> {
    fs::write(file, plaintext.unsecure_ref()).map_err(Err::Write)?;
    edit::edit_file(file).map_err(Err::Edit)?;
    Ok(fs::read(file).map_err(Err::Read)?.into())
}

/// Get directory to store files to edit in.
///
/// This attempts to use a secure directory if available, such as `/dev/shm` which doesn't store
/// anything on disk. Otherwise it defaults to the systems temporary directory.
fn dir() -> PathBuf {
    // Default to home directory on Windows due to notepad issues
    // Notepad, the default editor on Windows, is too retarded to save at the path we opened the
    // file at. Instead it always shows the 'Save As' dialog, no matter what, which defaults to the
    // users home folder. We'll just store the file there then...
    #[cfg(windows)]
    {
        if let Some(home) = env::home_dir() {
            return home;
        }
    }

    // Default to temporary dir
    #[allow(unused_mut)]
    let mut dir = env::temp_dir().into();

    // Prefer shared memory tmpfs if available so data won't leak to persistent disk
    #[cfg(unix)]
    {
        let dev_shm = PathBuf::from(UNIX_DIR_SHM);
        if dev_shm.is_dir() {
            dir = dev_shm;
        }
    }

    dir
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to create secure temporary file to edit data in")]
    Create(#[source] std::io::Error),

    #[error("failed to write data to temporary file to edit")]
    Write(#[source] std::io::Error),

    #[error("failed to open default editor to edit file")]
    Edit(#[source] std::io::Error),

    #[error("failed to read from edited file")]
    Read(#[source] std::io::Error),

    #[error("failed to close/remove temporary file, this may be a security issue")]
    Close(#[source] std::io::Error),
}
