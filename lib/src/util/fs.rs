use std::path::{Path, PathBuf};
#[cfg(all(feature = "tomb", target_os = "linux"))]
use std::process::{Command, Stdio};

use anyhow::Result;
#[cfg(all(feature = "tomb", target_os = "linux"))]
use fs_extra::dir::CopyOptions;
use thiserror::Error;

/// sudo binary.
#[cfg(all(feature = "tomb", target_os = "linux"))]
pub const SUDO_BIN: &str = crate::systemd_bin::SUDO_BIN;

/// chown binary.
pub const CHOWN_BIN: &str = "chown";

/// Calcualte directory size in bytes.
#[cfg(all(feature = "tomb", target_os = "linux"))]
pub fn dir_size(path: &Path) -> Result<u64, Err> {
    fs_extra::dir::get_size(path).map_err(Err::DirSize)
}

/// Copy contents of one directory to another.
///
/// This will only copy directory contents recursively. This will not copy the directory itself.
#[cfg(all(feature = "tomb", target_os = "linux"))]
pub fn copy_dir_contents(from: &Path, to: &Path) -> Result<()> {
    let mut options = CopyOptions::new();
    options.overwrite = true;
    options.copy_inside = true;
    options.content_only = true;
    Ok(fs_extra::dir::copy(from, to, &options)
        .map(|_| ())
        .map_err(Err::CopyDirContents)?)
}

/// Append a suffix to the filename of a path.
///
/// Errors if the path parent or file name could not be determined.
pub fn append_file_name(path: &Path, suffix: &str) -> Result<PathBuf> {
    Ok(path.parent().ok_or(Err::NoParent)?.join(format!(
        "{}{}",
        path.file_name().ok_or(Err::UnknownName)?.to_string_lossy(),
        suffix,
    )))
}

/// Chown a path to the current process' with `sudo`.
#[cfg(all(feature = "tomb", target_os = "linux"))]
pub(crate) fn sudo_chown(path: &Path, uid: u32, gid: u32, recursive: bool) -> Result<()> {
    // Build command
    let mut cmd = Command::new(SUDO_BIN);
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    cmd.arg("--");
    cmd.arg(CHOWN_BIN);
    if recursive {
        cmd.arg("--recursive");
    }
    cmd.arg(format!("{}:{}", uid, gid));
    cmd.arg(path);

    // Invoke and handle status
    let status = cmd.status().map_err(Err::SudoChown)?;
    if status.success() {
        Ok(())
    } else {
        Err(Err::Status(status).into())
    }
}

/// Chown a path to the current process' UID/GID with `sudo`.
#[cfg(all(feature = "tomb", target_os = "linux"))]
pub(crate) fn sudo_chown_current_user(path: &Path, recursive: bool) -> Result<()> {
    sudo_chown(
        path,
        nix::unistd::Uid::effective().as_raw(),
        nix::unistd::Gid::effective().as_raw(),
        recursive,
    )
}

#[derive(Debug, Error)]
pub enum Err {
    #[cfg(all(feature = "tomb", target_os = "linux"))]
    #[error("failed to measure directory size")]
    DirSize(#[source] fs_extra::error::Error),

    #[error("failed to copy directory contents")]
    #[cfg(all(feature = "tomb", target_os = "linux"))]
    CopyDirContents(#[source] fs_extra::error::Error),

    #[error("failed to append suffix to file path, unknown parent")]
    NoParent,

    #[error("failed to append suffix to file path, unknown name")]
    UnknownName,

    #[error("failed to invoke 'sudo chown' on path")]
    SudoChown(std::io::Error),

    #[error("system command exited with non-zero status code: {0}")]
    Status(std::process::ExitStatus),
}
