// TODO: rename this module to crypto, remove crypto.rs

#[cfg(feature = "crypto-gnupg-bin")]
pub mod gnupg_bin;
#[cfg(feature = "crypto-gpgme")]
pub mod gpgme;

use std::fs;
use std::path::Path;

use anyhow::Result;
use thiserror::Error;

use crate::types::{Ciphertext, Plaintext};
use crate::Recipients;

/// Cryptography type.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CryptoType {
    // TODO: rename to GnuPg_OpenPgp because we're using `~/.gnupg`?
    /// OpenPGP crypto.
    OpenPgp,
}

/// Get crypto context for given crypto type at runtime.
///
/// This selects a compatibel crypto context at runtime.
///
/// # Errors
///
/// Errors if no compatible crypto context is available because no backend is providing it. Also
/// errors if creating the context fails.
pub fn context(crypto: CryptoType) -> Result<Context, ContextErr> {
    // Select proper crypto backend
    match crypto {
        #[allow(unreachable_code)]
        CryptoType::OpenPgp => {
            #[cfg(feature = "crypto-gpgme")]
            return Ok(Context::from(Box::new(
                gpgme::context().map_err(|err| ContextErr::Create(err.into()))?,
            )));
            #[cfg(feature = "crypto-gnupg-bin")]
            return Ok(Context::from(Box::new(
                gnupg_bin::context().map_err(|err| ContextErr::Create(err.into()))?,
            )));
        }
    }

    Err(ContextErr::Unsupported(crypto))
}

/// Context wrapper.
///
/// Wraps a backend-specific context type and makes it easy to use.
pub struct Context {
    inner: Box<dyn ContextAdapter>,
}

impl Context {
    /// Construct new context from given context.
    fn from(context: Box<dyn ContextAdapter>) -> Self {
        Self { inner: context }
    }
}

impl Crypto for Context {}

impl Encrypt for Context {
    fn encrypt(&mut self, recipients: &Recipients, plaintext: Plaintext) -> Result<Ciphertext> {
        self.inner.encrypt(recipients, plaintext)
    }
}

impl Decrypt for Context {
    fn decrypt(&mut self, ciphertext: Ciphertext) -> Result<Plaintext> {
        self.inner.decrypt(ciphertext)
    }

    fn can_decrypt(&mut self, ciphertext: Ciphertext) -> Result<bool> {
        self.inner.can_decrypt(ciphertext)
    }
}

/// Crypto context.
///
/// Definese that a type is a crypto context adapter.
pub trait ContextAdapter: Crypto {}

pub trait Crypto: Encrypt + Decrypt {}

pub trait Encrypt {
    /// Encrypt the given plaintext.
    fn encrypt(&mut self, recipients: &Recipients, plaintext: Plaintext) -> Result<Ciphertext>;

    /// Encrypt the plaintext and write it to the file.
    fn encrypt_file(
        &mut self,
        recipients: &Recipients,
        plaintext: Plaintext,
        path: &Path,
    ) -> Result<()> {
        fs::write(path, self.encrypt(recipients, plaintext)?.unsecure_ref())
            .map_err(|err| EncryptErr::WriteFile(err).into())
    }
}

pub trait Decrypt {
    /// Decrypt the given ciphertext.
    fn decrypt(&mut self, ciphertext: Ciphertext) -> Result<Plaintext>;

    /// Decrypt the file at the given path.
    fn decrypt_file(&mut self, path: &Path) -> Result<Plaintext> {
        self.decrypt(fs::read(path).map_err(DecryptErr::ReadFile)?.into())
    }

    /// Check whether we can decrypt the given ciphertext.
    ///
    /// This checks whether we own the proper secret key to decrypt it.
    fn can_decrypt(&mut self, ciphertext: Ciphertext) -> Result<bool>;

    /// Check whether we can decrypt a file at the given path.
    fn can_decrypt_file(&mut self, path: &Path) -> Result<bool> {
        self.can_decrypt(fs::read(path).map_err(DecryptErr::ReadFile)?.into())
    }
}

#[derive(Debug, Error)]
pub enum ContextErr {
    #[error(
        "failed to obtain cryptography context, no backend available for {:?}",
        _0
    )]
    Unsupported(CryptoType),

    // TODO: keep original type here through adapter enum?
    #[error("failed to obtain cryptography context")]
    Create(#[source] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum EncryptErr {
    // TODO: is this used?
    #[error("failed to encrypt plaintext")]
    Encrypt(#[source] anyhow::Error),

    #[error("failed to write ciphertext to file")]
    WriteFile(#[source] std::io::Error),
}

#[derive(Debug, Error)]
pub enum DecryptErr {
    // TODO: is this used?
    #[error("failed to decrypt ciphertext")]
    Decrypt(#[source] anyhow::Error),

    #[error("failed to read ciphertext from file")]
    ReadFile(#[source] std::io::Error),
}

/// Prelude traits.
pub mod prelude {
    pub use super::{ContextAdapter, Crypto, Decrypt, Encrypt};
}
