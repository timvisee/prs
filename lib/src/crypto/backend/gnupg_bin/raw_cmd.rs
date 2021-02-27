//! Command helpers for raw interface.

use std::ffi::OsStr;
use std::io::Write;
use std::path::Path;
use std::process::{Command, ExitStatus, Output, Stdio};

use anyhow::Result;
use thiserror::Error;

// /// Invoke a gpg command with the given arguments.
// ///
// /// The command will take over the user console for in/output.
// pub(super) fn gpg<I, S>(bin: &Path, args: I) -> Result<()>
// where
//     I: IntoIterator<Item = S>,
//     S: AsRef<OsStr>,
// {
//     cmd_assert_status(cmd_gpg(bin, args).status().map_err(Err::System)?)
// }

/// Invoke a gpg command, returns output.
pub(super) fn gpg_output<I, S>(bin: &Path, args: I) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    cmd_gpg(bin, args)
        .output()
        .map_err(|err| Err::System(err).into())
}

/// Invoke a gpg command, returns output.
pub(super) fn gpg_stdin_output<I, S>(bin: &Path, args: I, stdin: &[u8]) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = cmd_gpg(bin, args);

    // Pass stdin to child process
    let mut child = cmd.spawn().unwrap();
    if let Err(err) = child.stdin.as_mut().unwrap().write_all(&stdin) {
        return Err(Err::System(err).into());
    }

    child
        .wait_with_output()
        .map_err(|err| Err::System(err).into())
}

/// Invoke a gpg command with the given arguments, return stdout on success.
pub(super) fn gpg_stdout_ok_bin<I, S>(bin: &Path, args: I) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = gpg_output(bin, args)?;
    cmd_assert_status(output.status)?;
    Ok(output.stdout)
}

/// Invoke a gpg command with the given arguments, return stdout on success.
pub(super) fn gpg_stdout_ok<I, S>(bin: &Path, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Ok(parse_text(&gpg_stdout_ok_bin(bin, args)?)
        .map_err(|err| Err::GpgCli(err.into()))?
        .trim()
        .into())
}

/// Invoke a gpg command with the given arguments, return stdout on success.
pub(super) fn gpg_stdin_stdout_ok_bin<I, S>(bin: &Path, args: I, stdin: &[u8]) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = gpg_stdin_output(bin, args, stdin)?;
    cmd_assert_status(output.status)?;
    Ok(output.stdout)
}

/// Build a gpg command to run.
fn cmd_gpg<I, S>(bin: &Path, args: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(bin);
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(args);
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

/// Try to parse command output bytes as text.
///
/// Command output formatting might not always be consistent. This function tries to parse both as
/// UTF-8 and UTF-16.
fn parse_text(bytes: &[u8]) -> Result<String, std::str::Utf8Error> {
    // Try to parse as UTF-8, remember error on failure
    let err = match std::str::from_utf8(bytes) {
        Ok(s) => return Ok(s.into()),
        Err(err) => err,
    };

    // Try to parse as UTF-16 on Windows
    #[cfg(windows)]
    if let Some(s) = u8_as_utf16(bytes) {
        return Ok(s);
    }

    Err(err)
}

/// Try to parse u8 slice as UTF-16 string.
#[cfg(windows)]
fn u8_as_utf16(bytes: &[u8]) -> Option<String> {
    // Bytes must be multiple of 2
    if bytes.len() % 2 != 0 {
        return None;
    }

    // Transmute to u16 slice, try to parse
    let bytes: &[u16] =
        unsafe { &std::slice::from_raw_parts(bytes.as_ptr() as *const u16, bytes.len() / 2) };
    String::from_utf16(bytes).ok()
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to complete gpg operation")]
    GpgCli(#[source] anyhow::Error),

    #[error("failed to invoke gpg command")]
    System(#[source] std::io::Error),

    #[error("gpg command exited with non-zero status code: {0}")]
    Status(std::process::ExitStatus),
}
