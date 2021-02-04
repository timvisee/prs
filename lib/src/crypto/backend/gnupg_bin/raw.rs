//! Raw interface to GPG binary.
//!
//! This provides the most basic and bare functions to interface with a GnuPG backend binary.

use std::collections::VecDeque;
use std::path::Path;

use anyhow::Result;
use regex::Regex;
use thiserror::Error;

use super::raw_cmd::{gpg_stdin_output, gpg_stdin_stdout_ok_bin, gpg_stdout_ok, gpg_stdout_ok_bin};
use crate::crypto::util;
use crate::types::{Ciphertext, Plaintext};

/// Partial output from gpg if the user does not own the secret key.
const GPG_OUTPUT_ERR_NO_SECKEY: &str = "decryption failed: No secret key";

/// Encrypt plaintext for the given recipients.
///
/// - `bin`: path to `gpg` binary
/// - `recipients`: list of recipient fingerprints to encrypt for
/// - `plaintext`: plaintext to encrypt
///
/// # Panics
///
/// Panics if list of recipients is empty.
pub fn encrypt(bin: &Path, recipients: &[&str], plaintext: Plaintext) -> Result<Ciphertext> {
    assert!(
        !recipients.is_empty(),
        "attempting to encrypt secret for empty list of recipients"
    );

    // Build argument list
    let mut args = vec!["--quiet", "--openpgp", "--trust-model", "always"];
    for fp in recipients {
        args.push("--recipient");
        args.push(fp);
    }
    args.push("--encrypt");

    Ok(Ciphertext::from(
        gpg_stdin_stdout_ok_bin(bin, args.as_slice(), plaintext.unsecure_ref())
            .map_err(|err| Err::Decrypt(err))?,
    ))
}

/// Decrypt ciphertext.
///
/// - `bin`: path to `gpg` binary
/// - `ciphertext`: ciphertext to decrypt
pub fn decrypt(bin: &Path, ciphertext: Ciphertext) -> Result<Plaintext> {
    // TODO: ensure ciphertext ends with PGP footer
    Ok(Plaintext::from(
        gpg_stdin_stdout_ok_bin(bin, &["--quiet", "--decrypt"], ciphertext.unsecure_ref())
            .map_err(|err| Err::Decrypt(err))?,
    ))
}

/// Check whether we can decrypt ciphertext.
///
/// This checks whether whether we own the secret key to decrypt the given ciphertext.
///
/// - `bin`: path to `gpg` binary
/// - `ciphertext`: ciphertext to check
// To check this, actual decryption is attempted, see this if this can be improved:
// https://stackoverflow.com/q/64633736/1000145
pub fn can_decrypt(bin: &Path, ciphertext: Ciphertext) -> Result<bool> {
    // TODO: ensure ciphertext ends with PGP footer

    let output = gpg_stdin_output(bin, &["--quiet", "--decrypt"], ciphertext.unsecure_ref())
        .map_err(|err| Err::Decrypt(err))?;

    match output.status.code() {
        Some(0) | None => Ok(true),
        Some(2) => Ok(!std::str::from_utf8(&output.stdout)?.contains(GPG_OUTPUT_ERR_NO_SECKEY)),
        Some(_) => Ok(true),
    }
}

/// Get all public keys from keychain.
///
/// - `bin`: path to `gpg` binary
pub fn public_keys(bin: &Path) -> Result<Vec<KeyId>> {
    let list = gpg_stdout_ok(bin, &["--list-keys", "--keyid-format", "LONG"]).map_err(Err::Keys)?;
    parse_key_list(list).ok_or_else(|| Err::UnexpectedOutput.into())
}

/// Get all private/secret keys from keychain.
///
/// - `bin`: path to `gpg` binary
pub fn private_keys(bin: &Path) -> Result<Vec<KeyId>> {
    let list =
        gpg_stdout_ok(bin, &["--list-secret-keys", "--keyid-format", "LONG"]).map_err(Err::Keys)?;
    parse_key_list(list).ok_or_else(|| Err::UnexpectedOutput.into())
}

/// Import given key from bytes into keychain.
///
/// - `bin`: path to `gpg` binary
///
/// # Panics
///
/// Panics if the provides key does not look like a public key.
pub fn import_key(bin: &Path, key: &[u8]) -> Result<()> {
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
    gpg_stdin_stdout_ok_bin(bin, &["--quiet", "--import"], key)
        .map(|_| ())
        .map_err(|err| Err::Import(err).into())
}

/// Export the given key as bytes.
///
/// # Panics
///
/// Panics if the received key does not look like a public key. This should never happen unless the
/// gpg binary backend is broken.
pub fn export_key(bin: &Path, fingerprint: &str) -> Result<Vec<u8>> {
    // Export key with gpg command
    let data = gpg_stdout_ok_bin(bin, &["--quiet", "--armor", "--export", fingerprint])
        .map_err(|err| Err::Export(err))?;

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

/// A key identifier with a fingerprint and user IDs.
#[derive(Clone)]
pub struct KeyId(pub String, pub Vec<String>);

/// Parse key list output from gnupg.
// TODO: throw proper errors on parse failure
fn parse_key_list(list: String) -> Option<Vec<KeyId>> {
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
                let fingerprint = util::format_fingerprint(lines.pop_front()?.trim());
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
                keys.push(KeyId(fingerprint, user_ids));
            }

            // Ignore empty lines
            l if l.trim().is_empty() => {}

            // Got something unexpected
            _ => return None,
        }
    }

    Some(keys)
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to communicate with gpg binary, got unexpected output")]
    UnexpectedOutput,

    #[error("failed to encrypt plaintext")]
    Encrypt(#[source] anyhow::Error),

    #[error("failed to decrypt ciphertext")]
    Decrypt(#[source] anyhow::Error),

    #[error("failed to obtain keys from gpg keychain")]
    Keys(#[source] anyhow::Error),

    #[error("failed to import key into gpg keychain")]
    Import(#[source] anyhow::Error),

    #[error("failed to export key from gpg keychain")]
    Export(#[source] anyhow::Error),
}
