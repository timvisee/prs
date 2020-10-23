use zeroize::Zeroize;

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
    pub fn first_line(self) -> Result<Plaintext, std::str::Utf8Error> {
        Ok(Plaintext(
            self.to_str()?.lines().next().unwrap().as_bytes().into(),
        ))
    }
}

impl From<&str> for Plaintext {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().into())
    }
}
