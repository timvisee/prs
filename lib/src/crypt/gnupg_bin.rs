use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;
use thiserror::Error;
use version_compare::Version;

use crate::types::{Ciphertext, Plaintext};
use crate::Recipients;

/// Binary name.
const BIN_NAME: &str = "gpg";

/// Minimum required version.
const VERSION_MIN: &str = "2.0.0";

// /// Protocol to use.
// const PROTO: Protocol = Protocol::OpenPgp;

// /// GPGME encryption flags.
// const ENCRYPT_FLAGS: EncryptFlags = EncryptFlags::ALWAYS_TRUST;

// TODO: also search this default path
// const DEFAULT_PATH: &str = "/usr/bin/gpg";

/// Create GnuPG binary context.
pub fn context() -> Result<Context, Err> {
    Ok(Context::from(find_gpg_bin()?))
}

/// Find the gpg binary.
fn find_gpg_bin() -> Result<PathBuf, Err> {
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

    // Exit code must be successful, stderr must be empty
    if !cmd_output.status.success() || !cmd_output.stderr.is_empty() {
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

impl super::ContextAdapter for Context {}

impl super::Crypto for Context {}

impl super::Encrypt for Context {
    /// Encrypt the given plaintext.
    fn encrypt(&mut self, recipients: &Recipients, plaintext: Plaintext) -> Result<Ciphertext> {
        unimplemented!()
    }
}

impl super::Decrypt for Context {
    /// Decrypt the given ciphertext.
    fn decrypt(&mut self, ciphertext: Ciphertext) -> Result<Plaintext> {
        unimplemented!()
    }

    fn can_decrypt(&mut self, ciphertext: Ciphertext) -> Result<bool> {
        unimplemented!()
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
    //
    // #[error("failed to encrypt plaintext")]
    // Encrypt(#[source] gpgme::Error),

    // #[error("failed to decrypt ciphertext")]
    // Decrypt(#[source] gpgme::Error),
}
