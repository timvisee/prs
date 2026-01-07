//! Common crypto utilities.

use anyhow::Result;

use super::{prelude::*, Config, Key};

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
    let fp = fingerprint.as_ref().trim_start();

    // Remove 0x prefix
    let fp = if fp.starts_with("0x") || fp.starts_with("0X") {
        &fp[2..]
    } else {
        fp
    };

    // Remove comment suffix
    let fp = fp.split('#').next().unwrap();

    // Trim whitespace and uppercase
    fp.trim_end().to_uppercase()
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

#[cfg(test)]
mod tests {
    #[rustfmt::skip]
    const FPS_NORMALIZE: &[(&str, &str)] = &[
        ("E2D8DE4D35CE386F",                                        "E2D8DE4D35CE386F"),
        ("364119B9",                                                "364119B9"        ),
        ("0xae78a4de7A738B54  ",                                    "AE78A4DE7A738B54"),
        ("0xB079912385023787 # username <user@example.com>",        "B079912385023787"),
        ("  0X47e3b6f9970b175f   # some comment # something else ", "47E3B6F9970B175F"),
    ];
    #[rustfmt::skip]
    const FPS_EQUAL: &[(&str, &str)] = &[
        // 8 characters is minimum
        ("AAAAAAAA",                                                "AAAAAAAA"),
        // Different casing
        ("e2d8de4d35CE386f",                                        "E2D8DE4d35CE386f"),
        // 0x prefixes
        ("0x364119B9",                                              "0X364119B9"),
        ("364119B9",                                                "0x364119B9"),
        // Comments
        ("364119B9 # comment 1",                                    "364119B9 # comment 2"),
        // Substrings
        ("AE78A4DE7A738B54",                                        "7A738B54"),
        ("AE78A4DE7A738B54",                                        "AE78A4DE"),
        ("   0xAE78A4DE7A738B54   # username <user@example.com>",   "a4de7a73"),
    ];
    #[rustfmt::skip]
    const FPS_NOT_EQUAL: &[(&str, &str)] = &[
        // Empty or too short fingerprints are never equal
        ("",                    ""),
        ("AAAAAAA",             "AAAAAAA"),
        ("AAAAAAAA",            "AAAAAAA"),
        ("AAAAAAA",             "AAAAAAAA"),
        // First is smaller than second
        ("364119B9",            "0364119B9"),
        ("0xae78a4de7A738B54",  "AE78A4DE7A738B540"),
    ];

    #[test]
    fn test_normalize_fingerprint() {
        for &(a, b) in FPS_NORMALIZE {
            assert_eq!(
                super::normalize_fingerprint(a),
                b,
                "{a:?} should normalize to {b:?}"
            );
        }
    }

    #[test]
    fn test_fingerprints_equal() {
        for &(a, b) in FPS_NORMALIZE {
            assert!(
                super::fingerprints_equal(a, b),
                "{a:?} and {b:?} should be equal",
            );
        }
        for &(a, b) in FPS_EQUAL {
            assert!(
                super::fingerprints_equal(a, b),
                "{a:?} and {b:?} should be equal",
            );
        }
        for &(a, b) in FPS_NOT_EQUAL {
            assert!(
                !super::fingerprints_equal(a, b),
                "{a:?} and {b:?} should not be equal",
            );
        }
    }
}
