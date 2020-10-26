use anyhow::Result;
use thiserror::Error;
use zeroize::Zeroize;

/// Newline character(s) on this platform.
#[cfg(not(windows))]
pub const NEWLINE: &str = "\n";
#[cfg(windows)]
pub const NEWLINE: &str = "\r\n";

/// Ciphertext.
///
/// Wraps ciphertext bytes. This type is limited on purpose, to prevent accidentally leaking the
/// ciphertext. The memory is explicitly zero'd when this is dropped.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct Ciphertext(pub Vec<u8>);

impl Ciphertext {
    /// New empty ciphertext.
    pub fn empty() -> Self {
        Self(vec![])
    }
}

/// Plaintext.
///
/// Wraps plaintext bytes. This type is limited on purpose, to prevent accidentally leaking the
/// plaintext. The memory is explicitly zero'd when this is dropped.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct Plaintext(pub Vec<u8>);

impl Plaintext {
    /// New empty plaintext.
    pub fn empty() -> Self {
        Self(vec![])
    }

    /// Construct plaintext from given string.
    pub fn from_string(text: String) -> Self {
        Self(text.into_bytes())
    }

    /// Get the plaintext as UTF8 string.
    // TODO: is this unsafe, because it might leak?
    pub fn to_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.0)
    }

    /// Get the first line of this secret as plaintext.
    ///
    /// Returns empty plaintext if there are no lines.
    pub fn first_line(self) -> Result<Plaintext> {
        Ok(Plaintext(
            self.to_str()
                .map_err(Err::FirstLine)?
                .lines()
                .next()
                .map(|l| l.as_bytes().into())
                .unwrap_or_else(|| vec![]),
        ))
    }

    /// Get all lines execpt the first one.
    ///
    /// Returns empty plaintext if there are no lines.
    pub fn except_first_line(self) -> Result<Plaintext> {
        Ok(Plaintext(
            self.to_str()
                .map_err(Err::FirstLine)?
                .lines()
                .skip(1)
                .collect::<Vec<&str>>()
                .join(NEWLINE)
                .into_bytes(),
        ))
    }

    /// Append other plaintext.
    ///
    /// Optionally adds platform newline.
    pub fn append(&mut self, other: Plaintext, newline: bool) {
        if newline {
            self.0.extend_from_slice(NEWLINE.as_bytes());
        }
        self.0.extend_from_slice(&other.0);
    }

    /// Check whether this plaintext is empty.
    ///
    /// - Empty if 0 bytes
    /// - Empty if bytes parsed as UTF-8 has trimmed length of 0 characters (ignored on encoding failure)
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
            || std::str::from_utf8(&self.0)
                .map(|s| s.trim().is_empty())
                .unwrap_or(false)
    }
}

impl From<&str> for Plaintext {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().into())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to select first line of plaintext")]
    FirstLine(#[source] std::str::Utf8Error),
}
