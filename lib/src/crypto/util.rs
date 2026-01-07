//! Common crypto utilities.

use anyhow::Result;

use super::{Config, Key, prelude::*};

/// Minimum hexadecimal length for a fingerprint to be considered valid
///
/// GnuPG requires at least 8 hexadecimal characters. GnuPG 2.2+ still allows 8 characters, but
/// emits a warning at least 16 characters are recommended.
const FINGERPRINT_MIN_LEN: usize = 8;

/// Format fingerprint in consistent format.
///
/// Trims and uppercases.
pub fn format_fingerprint<S: AsRef<str>>(fingerprint: S) -> String {
    normalize_fingerprint(fingerprint)
}

/// Normalize a fingerprint to some consistent format
///
/// Does the following in order:
/// - removes `0x` or `0X` prefix if present
/// - removes comment suffix if present
/// - trims whitespace
/// - uppercases
///
/// Does not normalize the length of the fingerprint
pub fn normalize_fingerprint<S: AsRef<str>>(fingerprint: S) -> String {
    let fp = fingerprint.as_ref();

    // Remove 0x prefix
    let fp = if fp.starts_with("0x") || fp.starts_with("0X") {
        &fp[2..]
    } else {
        fp
    };

    // Remove comment suffix
    let fp = fp.split('#').next().unwrap();

    // Trim whitespace and uppercase
    fp.trim().to_uppercase()
}

/// Check whether two fingerprints match
///
/// Normalizes both fingerprints using [`normalize_fingerprint`].
///
/// Returns true if `b` is a substring of `a`.
/// Requires at least [`FINGERPRINT_MIN_LEN`] hexadecimal characters in both.
pub fn fingerprints_equal<S: AsRef<str>, T: AsRef<str>>(a: S, b: T) -> bool {
    // Normalize both fingerprints
    let a = normalize_fingerprint(a);
    let b = normalize_fingerprint(b);

    // Require at least 8 characters
    if a.len() < FINGERPRINT_MIN_LEN || b.len() < FINGERPRINT_MIN_LEN {
        return false;
    }

    a.contains(&b)
}

/// Check whether a list of keys contains the given fingerprint.
pub fn keys_contain_fingerprint<S: AsRef<str>>(keys: &[Key], fingerprint: S) -> bool {
    keys.iter()
        .any(|key| fingerprints_equal(key.fingerprint(false), fingerprint.as_ref()))
}

/// Check whether the user has any private/secret key in their keychain.
pub fn has_private_key(config: &Config) -> Result<bool> {
    Ok(!super::context(config)?.keys_private()?.is_empty())
}
