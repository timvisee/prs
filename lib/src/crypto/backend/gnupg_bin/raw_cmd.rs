//! Command helpers for raw interface.

use std::char::decode_utf16;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::process::{Command, Output, Stdio};

use anyhow::Result;
use thiserror::Error;

use super::Config;
use crate::util;

/// Invoke a gpg command, returns output.
pub(super) fn gpg_output<I, S>(config: &Config, args: I) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    cmd_gpg(config, args)
        .output()
        .map_err(|err| Err::System(err).into())
}

/// Invoke a gpg command, returns output.
pub(super) fn gpg_stdin_output<I, S>(config: &Config, args: I, stdin: &[u8]) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = cmd_gpg(config, args);

    // Pass stdin to child process
    #[allow(clippy::zombie_processes)]
    let mut child = cmd.spawn().unwrap();
    if let Err(err) = child.stdin.as_mut().unwrap().write_all(stdin) {
        if let Err(err) = child.kill() {
            eprintln!("failed to kill gpg process: {err}");
        }
        return Err(Err::System(err).into());
    }

    child
        .wait_with_output()
        .map_err(|err| Err::System(err).into())
}

/// Invoke a gpg command with the given arguments, return stdout on success.
pub(super) fn gpg_stdout_ok_bin<I, S>(config: &Config, args: I) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = gpg_output(config, args)?;
    cmd_assert_status(config, &output)?;
    Ok(output.stdout)
}

/// Invoke a gpg command with the given arguments, return stdout on success.
pub(super) fn gpg_stdout_ok<I, S>(config: &Config, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Ok(parse_output(&gpg_stdout_ok_bin(config, args)?)
        .map_err(|err| Err::GpgCli(err.into()))?
        .trim()
        .into())
}

/// Invoke a gpg command with the given arguments, return stdout on success.
pub(super) fn gpg_stdin_stdout_ok_bin<I, S>(
    config: &Config,
    args: I,
    stdin: &[u8],
) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = gpg_stdin_output(config, args, stdin)?;
    cmd_assert_status(config, &output)?;
    Ok(output.stdout)
}

/// Build a gpg command to run.
fn cmd_gpg<I, S>(config: &Config, args: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    // TODO: select proper locale here, must be available on system
    // TODO: see: https://linuxconfig.org/how-to-list-all-available-locales-on-rhel7-linux

    let mut cmd = Command::new(&config.bin);
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("LANG", "en_US.UTF-8")
        .env("LANGUAGE", "en_US.UTF-8")
        // needed on some installations to force UTF-8 output, on debian:13 in Docker for example
        .arg("--display-charset")
        .arg("utf-8")
        .arg("--utf8-strings");
    if config.gpg_tty {
        cmd.arg("--pinentry-mode").arg("loopback");
        if !util::env::has_gpg_tty()
            && let Some(tty) = util::tty::get_tty()
        {
            cmd.env("GPG_TTY", tty);
        }
    }
    cmd.args(args);

    if config.verbose {
        log_cmd(&cmd);
    }

    cmd
}

/// Assert the exit status of a command based on its output.
///
/// On error, this prints stdout/stderr output in verbose mode.
///
/// Returns error is status is not succesful.
fn cmd_assert_status(config: &Config, output: &Output) -> Result<()> {
    if !output.status.success() {
        // Output stdout/stderr in verbose mode
        if config.verbose {
            if !output.stdout.is_empty() {
                let mut stdout = io::stdout();
                eprintln!("= gnupg stdout: ================");
                stdout
                    .write_all(&output.stdout)
                    .expect("failed to print gnupg stdout");
                let _ = stdout.flush();
                eprintln!("================================");
            }
            if !output.stderr.is_empty() {
                let mut stderr = io::stderr();
                eprintln!("= gnupg stderr: ================");
                stderr
                    .write_all(&output.stderr)
                    .expect("failed to print gnupg stderr");
                let _ = stderr.flush();
                eprintln!("================================");
            }
        }

        return Err(Err::Status(output.status).into());
    }

    Ok(())
}

/// Log the command to stderr.
// TODO: output working directory as well
// TODO: show stdin given to command
fn log_cmd(cmd: &Command) {
    let mut sh_cmd: Vec<String> = vec![];

    // Add environment variables
    sh_cmd.extend(cmd.get_envs().map(|(k, v)| {
        format!(
            "{}={}",
            shlex::try_quote(k.to_str().expect("gpg env key is not valid UTF-8"))
                .expect("failed to quite gpg env key"),
            shlex::try_quote(
                v.map(|v| v.to_str().expect("gpg env value is not valid UTF-8"))
                    .unwrap_or("")
            )
            .expect("failed to quite gpg env value"),
        )
    }));

    // Add binary name
    sh_cmd.push(
        shlex::try_quote(
            cmd.get_program()
                .to_str()
                .expect("gpg command binary is not valid UTF-8"),
        )
        .expect("failed to quote gpg command binary")
        .into(),
    );

    // Add program arguments
    sh_cmd.extend(cmd.get_args().map(|a| {
        shlex::try_quote(a.to_str().expect("gpg argument is not valid UTF-8"))
            .expect("failed to quote gpg command argument")
            .into()
    }));

    // Join invoked command into single string, and print
    let sh_cmd = sh_cmd
        .into_iter()
        .filter(|a| !a.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    eprintln!("$ {sh_cmd}");
}

/// Try to parse command output bytes as text.
///
/// Command output formatting might not always be consistent. This function tries to parse both as
/// UTF-8 and UTF-16.
fn parse_output(bytes: &[u8]) -> Result<String, std::str::Utf8Error> {
    // Try to parse as UTF-8, remember error on failure
    let err = match std::str::from_utf8(bytes) {
        Ok(s) => return Ok(s.into()),
        Err(err) => err,
    };

    // Try to parse as UTF-16
    if let Some(s) = u8_as_utf16(bytes) {
        return Ok(s);
    }

    Err(err)
}

/// Try to parse u8 slice as UTF-16 string.
fn u8_as_utf16(bytes: &[u8]) -> Option<String> {
    // Bytes must be multiple of 2
    if !bytes.len().is_multiple_of(2) {
        return None;
    }

    // Decode UTF-16 chars one by one and build a string
    let iter = (0..bytes.len() / 2).map(|i| u16::from_be_bytes([bytes[2 * i], bytes[2 * i + 1]]));
    decode_utf16(iter).collect::<Result<_, _>>().ok()
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
