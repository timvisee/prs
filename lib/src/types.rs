//! Secret plaintext and ciphertext types.

use anyhow::Result;
use secstr::SecVec;
use thiserror::Error;
use zeroize::Zeroize;

/// Delimiter for properties.
const PROPERTY_DELIMITER: char = ':';

/// Newline character(s) on this platform.
#[cfg(not(windows))]
pub const NEWLINE: &str = "\n";
#[cfg(windows)]
pub const NEWLINE: &str = "\r\n";

/// Ciphertext.
///
/// Wraps ciphertext bytes. This type is limited on purpose, to prevent accidentally leaking the
/// ciphertext. Security properties are enforced by `secstr::SecVec`.
pub struct Ciphertext(SecVec<u8>);

impl Ciphertext {
    /// New empty ciphertext.
    pub fn empty() -> Self {
        vec![].into()
    }

    /// Get unsecure reference to inner data.
    ///
    /// # Warning
    ///
    /// Unsecure because we cannot guarantee that the referenced data isn't cloned. Use with care!
    ///
    /// The reference itself is safe to use and share. Data may be cloned from this reference
    /// though, when that happens we lose track of it and are unable to securely handle it in
    /// memory. You should clone `Ciphertext` instead.
    pub(crate) fn unsecure_ref(&self) -> &[u8] {
        self.0.unsecure()
    }
}

impl From<Vec<u8>> for Ciphertext {
    fn from(mut other: Vec<u8>) -> Ciphertext {
        // Explicit zeroing of unsecure buffer required
        let into = Ciphertext(other.to_vec().into());
        other.zeroize();
        into
    }
}

/// Plaintext.
///
/// Wraps plaintext bytes. This type is limited on purpose, to prevent accidentally leaking the
/// plaintext. Security properties are enforced by `secstr::SecVec`.
#[derive(Clone, Eq, PartialEq)]
pub struct Plaintext(SecVec<u8>);

impl Plaintext {
    /// New empty plaintext.
    pub fn empty() -> Self {
        vec![].into()
    }

    /// Get unsecure reference to inner data.
    ///
    /// # Warning
    ///
    /// Unsecure because we cannot guarantee that the referenced data isn't cloned. Use with care!
    ///
    /// The reference itself is safe to use and share. Data may be cloned from this reference
    /// though, when that happens we lose track of it and are unable to securely handle it in
    /// memory. You should clone `Plaintext` instead.
    pub fn unsecure_ref(&self) -> &[u8] {
        self.0.unsecure()
    }

    /// Get the plaintext as UTF8 string.
    ///
    /// # Warning
    ///
    /// Unsecure because we cannot guarantee that the referenced data isn't cloned. Use with care!
    ///
    /// The reference itself is safe to use and share. Data may be cloned from this reference
    /// though, when that happens we lose track of it and are unable to securely handle it in
    /// memory. You should clone `Plaintext` instead.
    pub fn unsecure_to_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.unsecure_ref())
    }

    /// Get the first line of this secret as plaintext.
    ///
    /// Returns empty plaintext if there are no lines.
    pub fn first_line(self) -> Result<Plaintext> {
        Ok(self
            .unsecure_to_str()
            .map_err(Err::Utf8)?
            .lines()
            .next()
            .map(|l| l.as_bytes().into())
            .unwrap_or_else(|| vec![])
            .into())
    }

    /// Get all lines execpt the first one.
    ///
    /// Returns empty plaintext if there are no lines.
    pub fn except_first_line(self) -> Result<Plaintext> {
        Ok(self
            .unsecure_to_str()
            .map_err(Err::Utf8)?
            .lines()
            .skip(1)
            .collect::<Vec<&str>>()
            .join(NEWLINE)
            .into_bytes()
            .into())
    }

    /// Get line with the given property.
    ///
    /// Returns line with the given property. The property prefix is removed, and only the trimmed
    /// value is returned. Returns an error if the property does not exist.
    ///
    /// This will never return the first line being the password.
    pub fn property(self, property: &str) -> Result<Plaintext> {
        let property = property.trim().to_uppercase();
        self.unsecure_to_str()
            .map_err(Err::Utf8)?
            .lines()
            .skip(1)
            .find_map(|line| {
                let mut parts = line.splitn(2, PROPERTY_DELIMITER);
                if parts.next().unwrap().trim().to_uppercase() == property {
                    Some(parts.next().map(|value| value.trim()).unwrap_or("").into())
                } else {
                    None
                }
            })
            .ok_or_else(|| Err::Property(property.to_lowercase()).into())
    }

    /// Append other plaintext.
    ///
    /// Optionally adds platform newline.
    pub fn append(&mut self, other: Plaintext, newline: bool) {
        let mut data = self.unsecure_ref().to_vec();
        if newline {
            data.extend_from_slice(NEWLINE.as_bytes());
        }
        data.extend_from_slice(other.unsecure_ref());
        self.0 = data.into();
    }

    /// Check whether this plaintext is empty.
    ///
    /// - Empty if 0 bytes
    /// - Empty if bytes parsed as UTF-8 has trimmed length of 0 characters (ignored on encoding failure)
    pub fn is_empty(&self) -> bool {
        self.unsecure_ref().is_empty()
            || std::str::from_utf8(self.unsecure_ref())
                .map(|s| s.trim().is_empty())
                .unwrap_or(false)
    }
}

impl From<String> for Plaintext {
    fn from(mut other: String) -> Plaintext {
        // Explicit zeroing of unsecure buffer required
        let into = Plaintext(other.as_bytes().into());
        other.zeroize();
        into
    }
}

impl From<Vec<u8>> for Plaintext {
    fn from(mut other: Vec<u8>) -> Plaintext {
        // Explicit zeroing of unsecure buffer required
        let into = Plaintext(other.to_vec().into());
        other.zeroize();
        into
    }
}

impl From<&str> for Plaintext {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().into())
    }
}

/// A plaintext or ciphertext handling error.
#[derive(Debug, Error)]
pub enum Err {
    #[error("failed parse plaintext as UTF-8")]
    Utf8(#[source] std::str::Utf8Error),

    #[error("property '{}' does not exist in plaintext", _0)]
    Property(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plaintext_empty() {
        let empty = Plaintext::empty();
        assert!(empty.is_empty(), "empty plaintext should be empty");
    }

    #[test]
    fn plaintext_is_empty() {
        // Test empty
        let mut plaintext = Plaintext::from("");
        assert!(plaintext.is_empty(), "empty plaintext should be empty");
        assert!(
            plaintext.unsecure_ref().is_empty(),
            "empty plaintext should be empty"
        );

        // Test not empty
        plaintext.append(Plaintext::from("abc"), false);
        assert!(!plaintext.is_empty(), "empty plaintext should not be empty");
        assert!(
            !plaintext.unsecure_ref().is_empty(),
            "empty plaintext should not be empty"
        );
    }

    #[test]
    fn plaintext_first_line() {
        // (input, output)
        let set = vec![
            ("", ""),
            ("\n", ""),
            ("abc", "abc"),
            ("abc\n", "abc"),
            ("abc\ndef\r\nghi", "abc"),
            ("abc\r\ndef\nghi", "abc"),
        ];

        for (input, output) in set {
            assert_eq!(
                Plaintext::from(input)
                    .first_line()
                    .unwrap()
                    .unsecure_to_str()
                    .unwrap(),
                output,
                "first line of plaintext is incorrect",
            );
        }
    }

    #[test]
    fn plaintext_except_first_line() {
        // (input, output)
        let set = vec![
            ("", ""),
            ("\n", ""),
            ("abc", ""),
            ("abc\n", ""),
            ("abc\ndef\r\nghi", "def\nghi"),
            ("abc\r\ndef\nghi", "def\nghi"),
        ];

        for (input, output) in set {
            assert_eq!(
                Plaintext::from(input)
                    .except_first_line()
                    .unwrap()
                    .unsecure_to_str()
                    .unwrap(),
                output,
                "first line of plaintext is incorrect",
            );
        }
    }

    #[test]
    fn plaintext_append() {
        // Append to empty without newline
        let mut plaintext = Plaintext::empty();
        plaintext.append(Plaintext::from("abc"), false);
        assert_eq!(plaintext.unsecure_to_str().unwrap(), "abc");
        plaintext.append(Plaintext::from("def"), false);
        assert_eq!(plaintext.unsecure_to_str().unwrap(), "abcdef");

        // Append to empty with newline
        let mut plaintext = Plaintext::empty();
        plaintext.append(Plaintext::from("abc"), true);
        assert_eq!(plaintext.unsecure_to_str().unwrap(), "\nabc");
        plaintext.append(Plaintext::from("def"), true);
        assert_eq!(plaintext.unsecure_to_str().unwrap(), "\nabc\ndef");

        // Append empty to empty
        let mut plaintext = Plaintext::empty();
        plaintext.append(Plaintext::empty(), false);
        assert!(plaintext.is_empty());
        plaintext.append(Plaintext::empty(), true);
        assert_eq!(plaintext.unsecure_to_str().unwrap(), "\n");

        // Keep existing newlines
        let mut plaintext = Plaintext::from("\n\n");
        plaintext.append(Plaintext::from("\n\n"), false);
        assert_eq!(plaintext.unsecure_to_str().unwrap(), "\n\n\n\n");
        plaintext.append(Plaintext::from("\n\n"), true);
        assert_eq!(plaintext.unsecure_to_str().unwrap(), "\n\n\n\n\n\n\n");
    }

    #[quickcheck]
    fn plaintext_append_string(a: String, b: String, c: String) {
        // Appending lots of random stuff and parsing as string should never fail
        let mut plaintext = Plaintext::from(a);
        plaintext.append(Plaintext::from(b), false);
        plaintext.append(Plaintext::from(c), true);
        plaintext.unsecure_to_str().unwrap();
    }

    #[test]
    fn plaintext_property() {
        // Never select property from first line, but do from others
        assert!(
            Plaintext::from("Name: abc").property("name").is_err(),
            "should never select property from first line"
        );
        assert_eq!(
            Plaintext::from("Name: abc\nName: def")
                .property("name")
                .unwrap()
                .unsecure_to_str()
                .unwrap(),
            "def",
            "should select property value from all but the first line"
        );

        // (input, property to select, output)
        #[rustfmt::skip]
        let set = vec![
            // Nothing/empty
            ("", "", None),

            // Properties
            ("\nName: abc", "Name", Some("abc")),
            ("\n   Name   :   abc   ", "Name", Some("abc")),
            ("\nName: abc\nName: def", "Name", Some("abc")),
            ("\nName: abc\nMail: abc@example.com", "Mail", Some("abc@example.com")),
            ("\nName: abc\nMail: abc@example.com", "Name", Some("abc")),

            // Empty property
            ("\nEmpty:", "Empty", Some("")),
            ("\nEmpty:   ", "Empty", Some("")),

            // Missing
            ("\nName: abc\nMail: abc@example.com", "missing", None),

            // Capitalization
            ("\nName: abc", "name", Some("abc")),
            ("\nName: abc", "NAME", Some("abc")),
            ("\nName: abc", "nAME", Some("abc")),
            ("\nNAME: abc", "name", Some("abc")),
            ("\nnAmE: abc", "name", Some("abc")),
            ("\nNAME: abc\nname: def", "name", Some("abc")),
        ];

        for (input, property, output) in set {
            let val = Plaintext::from(input).property(property).ok();
            if let Some(output) = output {
                assert_eq!(
                    val.unwrap().unsecure_to_str().unwrap(),
                    output,
                    "incorrect property value",
                );
            } else {
                assert!(val.is_none(), "no property should be selected",);
            }
        }
    }

    #[quickcheck]
    fn plaintext_must_zero_on_drop(plaintext: String) -> bool {
        // Skip all-zero/empty because we cannot reliably test
        if plaintext.len() < 16 || plaintext.bytes().all(|b| b == 0) {
            return true;
        }

        // Create plaintext, remember memory range and data, then drop plaintext
        let plaintext = Plaintext::from(plaintext);
        let must_not_match = plaintext.0.unsecure().to_vec();
        let range = plaintext.0.unsecure().as_ptr_range();
        drop(plaintext);

        // Retake same slice of memory that we've dropped
        let slice: &[u8] = unsafe {
            std::slice::from_raw_parts(range.start, range.end as usize - range.start as usize)
        };

        // Memory must have been explicitly zeroed, it must never be the same as before
        slice != &must_not_match
    }

    #[test]
    fn ciphertext_empty() {
        let empty = Ciphertext::empty();
        assert!(
            empty.unsecure_ref().is_empty(),
            "empty ciphertext should be empty"
        );
    }

    #[quickcheck]
    fn ciphertext_must_zero_on_drop(ciphertext: Vec<u8>) -> bool {
        // Skip all-zero/empty because we cannot reliably test
        if ciphertext.len() < 16 || ciphertext.iter().all(|b| *b == 0) {
            return true;
        }

        // Create ciphertext, remember memory range and data, then drop ciphertext
        let ciphertext = Ciphertext::from(ciphertext);
        let must_not_match = ciphertext.0.unsecure().to_vec();
        let range = ciphertext.0.unsecure().as_ptr_range();
        drop(ciphertext);

        // Retake same slice of memory that we've dropped
        let slice: &[u8] = unsafe {
            std::slice::from_raw_parts(range.start, range.end as usize - range.start as usize)
        };

        // Memory must have been explicitly zeroed, it must never be the same as before
        slice != &must_not_match
    }
}
