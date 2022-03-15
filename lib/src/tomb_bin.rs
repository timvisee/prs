use std::env;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, ExitStatus};

use anyhow::Result;
use thiserror::Error;

use crate::crypto::Key;
use crate::util;

/// Binary name.
pub const BIN_NAME: &str = "tomb";

/// Invoke tomb dig.
///
/// `mbs` is the size of the tomb to create in megabytes.
pub fn tomb_dig(tomb_file: &Path, mbs: u32, settings: TombSettings) -> Result<()> {
    tomb(
        &[
            "dig",
            tomb_file
                .to_str()
                .expect("tomb path has invalid UTF-8 characters"),
            "-s",
            &format!("{}", mbs),
        ],
        settings,
    )
}

/// Invoke tomb forge.
pub fn tomb_forge(key_file: &Path, key: &Key, settings: TombSettings) -> Result<()> {
    tomb(
        &[
            "forge",
            key_file
                .to_str()
                .expect("tomb key path has invalid UTF-8 characters"),
            "-gr",
            &key.fingerprint(false),
        ],
        settings,
    )
}

/// Invoke tomb lock.
pub fn tomb_lock(
    tomb_file: &Path,
    key_file: &Path,
    key: &Key,
    settings: TombSettings,
) -> Result<()> {
    tomb(
        &[
            "lock",
            tomb_file
                .to_str()
                .expect("tomb path has invalid UTF-8 characters"),
            "-k",
            key_file
                .to_str()
                .expect("tomb key path has invalid UTF-8 characters"),
            "-gr",
            &key.fingerprint(false),
        ],
        settings,
    )
}

/// Invoke tomb open.
pub fn tomb_open(
    tomb_file: &Path,
    key_file: &Path,
    store_dir: &Path,
    key: Option<&Key>,
    settings: TombSettings,
) -> Result<()> {
    // Build command arguments list
    let key_fp = key.map(|key| key.fingerprint(false));
    let mut args = vec![
        "open",
        tomb_file
            .to_str()
            .expect("tomb path contains invalid UTF-8"),
        "-k",
        key_file
            .to_str()
            .expect("tomb key path contains invalid UTF-8"),
        "-p",
    ];
    match &key_fp {
        Some(fp) => args.extend(&["-gr", fp]),
        None => args.extend(&["-g"]),
    }
    args.push(
        store_dir
            .to_str()
            .expect("password store directory path contains invalid UTF-8"),
    );

    // TODO: ensure tomb file, key and store dir exist
    tomb(&args, settings)
}

/// Invoke tomb close.
pub fn tomb_close(tomb_file: &Path, settings: TombSettings) -> Result<()> {
    tomb(
        &["close", name(tomb_file).expect("failed to get tomb name")],
        settings,
    )
}

/// Invoke tomb resize.
pub fn tomb_resize(
    tomb_file: &Path,

    key_file: &Path,
    size_mb: u32,
    settings: TombSettings,
) -> Result<()> {
    tomb(
        &[
            "resize",
            tomb_file
                .to_str()
                .expect("tomb path contains invalid UTF-8"),
            "-k",
            key_file
                .to_str()
                .expect("tomb key path contains invalid UTF-8"),
            "-g",
            "-s",
            &format!("{}", size_mb),
        ],
        settings,
    )
}

/// Get tomb name based on path.
pub fn name(path: &Path) -> Option<&str> {
    path.file_name()?.to_str()?.rsplitn(2, '.').last()
}

/// Invoke a tomb command with the given arguments.
///
/// The command will take over the user console for in/output.
fn tomb<I, S>(args: I, settings: TombSettings) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    cmd_assert_status(cmd_tomb(args, settings).status().map_err(Err::Tomb)?)
}

/// Build a tomb command to run.
fn cmd_tomb<I, S>(args: I, settings: TombSettings) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    // Build base command
    let mut cmd = if let Ok(bin) = env::var("PASSWORD_STORE_TOMB") {
        Command::new(bin)
    } else {
        Command::new(BIN_NAME)
    };

    // Explicitly set GPG_TTY on Wayland to current stdin if not set
    // Fixed GPG not showing pinentry prompt properly on Wayland
    // Issue: https://github.com/timvisee/prs/issues/8#issuecomment-871090949
    if util::env::is_wayland() && !util::env::has_gpg_tty() {
        if let Some(tty) = util::tty::get_tty() {
            cmd.env("GPG_TTY", tty);
        }
    }

    // Set global flags, add arguments
    if settings.quiet {
        cmd.arg("-q");
    }
    if settings.verbose {
        cmd.arg("-D");
    }
    if settings.force {
        cmd.arg("-f");
    }
    cmd.args(args);

    cmd
}

/// Assert the exit status of a command.
///
/// Returns error is status is not succesful.
fn cmd_assert_status(status: ExitStatus) -> Result<()> {
    if !status.success() {
        return Err(Err::Status(status).into());
    }
    Ok(())
}

/// Tomb command settings.
#[derive(Copy, Clone)]
pub struct TombSettings {
    /// Run in quiet (-q) mode.
    pub quiet: bool,

    /// Run in verbose (-D) mode.
    pub verbose: bool,

    /// Run with force (-f).
    pub force: bool,
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to invoke tomb command")]
    Tomb(#[source] std::io::Error),

    #[error("tomb operation exited with non-zero status code: {0}")]
    Status(std::process::ExitStatus),
}
