use std::collections::VecDeque;
use std::ffi::OsStr;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};

use anyhow::Result;
use regex::Regex;
use thiserror::Error;
use version_compare::Version;

use super::prelude::*;
use crate::types::{Ciphertext, Plaintext};
use crate::Recipients;

/// Binary name.
const BIN_NAME: &str = "gpg";

/// Minimum required version.
const VERSION_MIN: &str = "2.0.0";

/// Partial output from gpg if the user does not own the secret key.
const GPG_OUTPUT_ERR_NO_SECKEY: &str = "decryption failed: No secret key";

/// Create GnuPG binary context.
pub fn context() -> Result<Context, Err> {
    Ok(Context::from(find_gpg_bin()?))
}

/// Find the gpg binary.
fn find_gpg_bin() -> Result<PathBuf, Err> {
    // TODO: if not found, try default path
    let path = which::which(BIN_NAME).map_err(Err::Unavailable)?;
    test_gpg_compat(&path)?;
    Ok(path)
}

/// Test gpg binary compatibility.
fn test_gpg_compat(path: &Path) -> Result<(), Err> {
    let cmd_output = Command::new(&path)
        .arg("--version")
        .output()
        .map_err(|err| Err::Binary(err))?;

    // Exit code must be successful
    if !cmd_output.status.success() {
        return Err(Err::UnexpectedOutput);
    }

    // Strip stdout to just the version number
    let stdout = std::str::from_utf8(cmd_output.stdout.as_slice())
        .ok()
        .and_then(|stdout| stdout.trim_start().lines().next())
        .and_then(|stdout| stdout.trim().strip_prefix("gpg (GnuPG) "))
        .map(|stdout| stdout.trim())
        .ok_or(Err::UnexpectedOutput)?;

    // Assert minimum version number
    let ver_min = Version::from(VERSION_MIN).unwrap();
    let ver_gpg = Version::from(stdout).unwrap();
    if ver_gpg < ver_min {
        return Err(Err::UnsupportedVersion(ver_gpg.to_string()));
    }

    Ok(())
}

/// GnuPG binary crypto context.
pub struct Context {
    /// Binary path.
    bin: PathBuf,
}

impl Context {
    /// Construct context from binary path.
    fn from(path: PathBuf) -> Self {
        Self { bin: path }
    }
}

impl ContextAdapter for Context {}

impl Crypto for Context {
    fn keychain<'a>(&'a mut self) -> Box<dyn IsKeychain + 'a> {
        Box::new(Keychain::from(self))
    }
}

impl Encrypt for Context {
    fn encrypt(&mut self, recipients: &Recipients, plaintext: Plaintext) -> Result<Ciphertext> {
        // TODO: list of recipients must not be empty

        // Build argument list
        let mut args = vec![
            "--quiet".into(),
            "--openpgp".into(),
            "--trust-model".into(),
            "always".into(),
        ];
        for recip in recipients.keys() {
            args.push("--recipient".into());
            args.push(recip.fingerprint(false));
        }
        args.push("--encrypt".into());

        Ok(Ciphertext::from(
            gpg_stdin_stdout_ok_bin(&self.bin, args.as_slice(), plaintext.unsecure_ref())
                .map_err(|err| Err::Decrypt(err))?,
        ))
    }
}

impl Decrypt for Context {
    fn decrypt(&mut self, ciphertext: Ciphertext) -> Result<Plaintext> {
        // TODO: ensure ciphertext ends with PGP footer
        Ok(Plaintext::from(
            gpg_stdin_stdout_ok_bin(
                &self.bin,
                &["--quiet", "--decrypt"],
                ciphertext.unsecure_ref(),
            )
            .map_err(|err| Err::Decrypt(err))?,
        ))
    }

    fn can_decrypt(&mut self, ciphertext: Ciphertext) -> Result<bool> {
        // TODO: ensure ciphertext ends with PGP footer

        let output = gpg_stdin_output(
            &self.bin,
            &["--quiet", "--decrypt"],
            ciphertext.unsecure_ref(),
        )
        .map_err(|err| Err::Decrypt(err))?;

        match output.status.code() {
            Some(0) | None => Ok(true),
            Some(2) => Ok(!std::str::from_utf8(&output.stdout)?.contains(GPG_OUTPUT_ERR_NO_SECKEY)),
            Some(_) => Ok(true),
        }
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to find GnuPG gpg binary")]
    Unavailable(#[source] which::Error),

    #[error("failed to communicate with GnuPG gpg binary")]
    Binary(#[source] std::io::Error),

    #[error("failed to communicate with GnuPG gpg binary, got unexpected output")]
    UnexpectedOutput,

    #[error("failed to use GnuPG gpg binary, unsupported version: {}", _0)]
    UnsupportedVersion(String),

    // TODO: is this used?
    #[error("failed to obtain GPGME cryptography context")]
    Context(#[source] gpgme::Error),

    #[error("failed to encrypt plaintext")]
    Encrypt(#[source] anyhow::Error),

    #[error("failed to decrypt ciphertext")]
    Decrypt(#[source] anyhow::Error),

    #[error("failed to complete gpg operation")]
    Other(#[source] std::io::Error),

    #[error("failed to complete gpg operation")]
    GpgCli(#[source] anyhow::Error),

    #[error("failed to invoke system command")]
    System(#[source] std::io::Error),

    #[error("system command exited with non-zero status code: {0}")]
    Status(std::process::ExitStatus),
}

#[derive(Debug, Error)]
pub enum KeychainErr {
    #[error("failed to communicate with GnuPG gpg binary, got unexpected output")]
    UnexpectedOutput,

    #[error("failed to list keys")]
    Keys(#[source] anyhow::Error),

    #[error("failed to import key")]
    Import(#[source] anyhow::Error),

    #[error("failed to export key")]
    Export(#[source] anyhow::Error),
}

/// Invoke a gpg command with the given arguments.
///
/// The command will take over the user console for in/output.
fn gpg<I, S>(bin: &Path, args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    cmd_assert_status(cmd_gpg(bin, args).status().map_err(Err::System)?)
}

/// Invoke a gpg command, returns output.
fn gpg_output<I, S>(bin: &Path, args: I) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    cmd_gpg(bin, args)
        .output()
        .map_err(|err| Err::System(err).into())
}

/// Invoke a gpg command, returns output.
fn gpg_stdin_output<I, S>(bin: &Path, args: I, stdin: &[u8]) -> Result<Output>
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
fn gpg_stdout_ok_bin<I, S>(bin: &Path, args: I) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = gpg_output(bin, args)?;
    cmd_assert_status(output.status)?;
    Ok(output.stdout)
}

/// Invoke a gpg command with the given arguments, return stdout on success.
fn gpg_stdout_ok<I, S>(bin: &Path, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Ok(std::str::from_utf8(&gpg_stdout_ok_bin(bin, args)?)
        .map_err(|err| Err::GpgCli(err.into()))?
        .trim()
        .into())
}

/// Invoke a gpg command with the given arguments, return stdout on success.
fn gpg_stdin_stdout_ok_bin<I, S>(bin: &Path, args: I, stdin: &[u8]) -> Result<Vec<u8>>
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

    // Debug invoked gpg commands
    // eprintln!("Invoked: {:?}", &cmd);

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

/// Provides access to GPGME keys.
pub struct Keychain<'a> {
    context: &'a Context,
}

impl<'a> Keychain<'a> {
    fn from(context: &'a Context) -> Self {
        Self { context }
    }
}

impl<'a> IsKeychain for Keychain<'a> {
    fn keys_public(&mut self) -> Result<Vec<Box<dyn IsKey>>> {
        let list = gpg_stdout_ok(
            &self.context.bin,
            &["--list-keys", "--keyid-format", "LONG"],
        )
        .map_err(KeychainErr::Keys)?;
        Ok(parse_key_list(list)
            .ok_or(Err::UnexpectedOutput)?
            .into_iter()
            .map(|key| Box::new(key) as Box<dyn IsKey>)
            .collect())
    }

    fn keys_private(&mut self) -> Result<Vec<Box<dyn IsKey>>> {
        let list = gpg_stdout_ok(
            &self.context.bin,
            &["--list-secret-keys", "--keyid-format", "LONG"],
        )
        .map_err(KeychainErr::Keys)?;
        Ok(parse_key_list(list)
            .ok_or(Err::UnexpectedOutput)?
            .into_iter()
            .map(|key| Box::new(key) as Box<dyn IsKey>)
            .collect())
    }

    /// Import the given key from bytes.
    fn import_key(&mut self, key: &[u8]) -> Result<()> {
        // Assert we're importing a public key
        let key_str = std::str::from_utf8(&key).expect("exported key is invalid UTF-8");
        assert!(
            !key_str.contains("PRIVATE KEY"),
            "imported key contains PRIVATE KEY, blocked to prevent accidentally leaked secret key"
        );
        assert!(
            key_str.contains("PUBLIC KEY"),
            "imported key must contain PUBLIC KEY, blocked to prevent accidentally leaked secret key"
        );

        // Import key with gpg command
        gpg_stdin_stdout_ok_bin(&self.context.bin, &["--quiet", "--import"], key)
            .map(|_| ())
            .map_err(|err| KeychainErr::Import(err).into())
    }

    /// Export the given key as bytes.
    // TODO: do not box, just give key
    fn export_key(&mut self, key: &Box<dyn IsKey>) -> Result<Vec<u8>> {
        // Export key with gpg command
        let data = gpg_stdout_ok_bin(
            &self.context.bin,
            &["--quiet", "--armor", "--export", &key.fingerprint(false)],
        )
        .map_err(|err| KeychainErr::Export(err))?;

        // Assert we're exporting a public key
        let data_str = std::str::from_utf8(&data).expect("exported key is invalid UTF-8");
        assert!(
            !data_str.contains("PRIVATE KEY"),
            "exported key contains PRIVATE KEY, blocked to prevent accidentally leaking secret key"
        );
        assert!(
            data_str.contains("PUBLIC KEY"),
            "exported key must contain PUBLIC KEY, blocked to prevent accidentally leaking secret key"
        );

        Ok(data)
    }
}

/// Parse key list output from gnupg.
// TODO: throw proper errors on parse failure
fn parse_key_list(list: String) -> Option<Vec<Key>> {
    let mut lines: VecDeque<_> = list.lines().collect();

    // Second line must be a line
    lines.pop_front()?;
    if lines
        .pop_front()?
        .bytes()
        .filter(|&b| b != b'-')
        .take(1)
        .count()
        > 0
    {
        return None;
    }

    let re_fingerprint = Regex::new(r"^[0-9A-F]{16,}$").unwrap();
    let re_user_id = Regex::new(r"^uid\s*\[[a-z ]+\]\s*(.*)$").unwrap();

    // Walk through the list, collect list of keys
    let mut keys = Vec::new();
    while !lines.is_empty() {
        match lines.pop_front()? {
            // Start reading a new key
            l if l.starts_with("pub ") || l.starts_with("sec ") => {
                // Get the fingerprint
                let fingerprint = super::format_fingerprint(lines.pop_front()?.trim());
                if !re_fingerprint.is_match(&fingerprint) {
                    return None;
                }

                // Find and parse user IDs
                let mut user_ids = Vec::new();
                while !lines.is_empty() {
                    match lines.pop_front()? {
                        // Read user ID
                        l if l.starts_with("uid ") => {
                            let captures = re_user_id.captures(l)?;
                            user_ids.push(captures[1].to_string());
                        }

                        // Finalize on empty line
                        l if l.trim().is_empty() => break,

                        _ => {}
                    }
                }

                // Add read key to list
                keys.push(Key {
                    fingerprint,
                    user_ids,
                });
            }

            // Ignore empty lines
            l if l.trim().is_empty() => {}

            // Got something unexpected
            _ => return None,
        }
    }

    Some(keys)
}

/// GnuPG binary key, a recipient.
#[derive(Clone)]
pub struct Key {
    fingerprint: String,
    user_ids: Vec<String>,
}

impl IsKey for Key {
    /// Get fingerprint.
    fn fingerprint(&self, short: bool) -> String {
        super::format_fingerprint(if short {
            self.fingerprint[self.fingerprint.len() - 16..].to_string()
        } else {
            self.fingerprint.clone()
        })
    }

    /// Format user data to displayable string.
    fn user_display(&self) -> String {
        // TODO: this should never be empty!
        self.user_ids.join("; ")
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.fingerprint == other.fingerprint
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[GPG] {} - {}",
            self.fingerprint(true),
            self.user_display()
        )
    }
}
