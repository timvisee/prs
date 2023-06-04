//! Common crypto utilities.

use anyhow::Result;

use super::{Config, Key, prelude::*};

/// Format fingerprint in consistent format.
///
/// Trims and uppercases.
pub fn format_fingerprint<S: AsRef<str>>(fingerprint: S) -> String {
    fingerprint.as_ref().trim().to_uppercase()
}

/// Check whether two fingerprints match.
pub fn fingerprints_equal<S: AsRef<str>, T: AsRef<str>>(a: S, b: T) -> bool {
    !a.as_ref().trim().is_empty()
        && a.as_ref().trim().to_uppercase().contains(&b.as_ref().trim().to_uppercase())
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
