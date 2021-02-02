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

impl Crypto for Context {
    fn keychain<'a>(&'a mut self) -> Box<dyn IsKeychain + 'a> {
        self.inner.keychain()
    }
}

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

pub trait Crypto: Encrypt + Decrypt {
    /// Get keychain for this crypto backend.
    fn keychain<'a>(&'a mut self) -> Box<dyn IsKeychain + 'a>;
}

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

/// Provides access to crypto backend keys.
pub struct Keychain<'a> {
    inner: &'a mut Box<dyn IsKeychain>,
}

impl<'a> Keychain<'a> {
    fn from(inner: &'a mut Box<dyn IsKeychain>) -> Self {
        Self { inner }
    }
}

impl<'a> IsKeychain for Keychain<'a> {
    fn keys_public(&mut self) -> Result<Vec<Box<dyn IsKey>>> {
        self.inner.keys_public()
    }

    fn keys_private(&mut self) -> Result<Vec<Box<dyn IsKey>>> {
        self.inner.keys_private()
    }

    /// Import the given key from bytes.
    fn import_key(&mut self, key: &[u8]) -> Result<()> {
        self.inner.import_key(key)
    }

    /// Export the given key as bytes.
    fn export_key(&mut self, key: &Box<dyn IsKey>) -> Result<Vec<u8>> {
        self.inner.export_key(key)
    }
}

pub trait IsKeychain {
    /// Get all public keys.
    fn keys_public(&mut self) -> Result<Vec<Box<dyn IsKey>>>;

    /// Get all private keys.
    fn keys_private(&mut self) -> Result<Vec<Box<dyn IsKey>>>;

    /// Import the given key from bytes.
    fn import_key(&mut self, key: &[u8]) -> Result<()>;

    /// Import the given key from a file.
    fn import_key_file(&mut self, path: &Path) -> Result<()> {
        self.import_key(&fs::read(path).map_err(KeychainErr::ReadKey)?)
            .map_err(|err| KeychainErr::Import(err).into())
    }

    /// Export the given key as bytes.
    fn export_key(&mut self, key: &Box<dyn IsKey>) -> Result<Vec<u8>>;

    /// Export the given key to a file.
    fn export_key_file(&mut self, key: &Box<dyn IsKey>, path: &Path) -> Result<()> {
        fs::write(path, self.export_key(key).map_err(KeychainErr::Export)?)
            .map_err(|err| KeychainErr::WriteKey(err).into())
    }
}

pub struct Key {
    key: Box<dyn IsKey>,
}

impl IsKey for Key {
    fn fingerprint(&self, short: bool) -> String {
        self.key.fingerprint(short)
    }

    /// Format user data to displayable string.
    fn user_display(&self) -> String {
        self.key.user_display()
    }
}

pub trait IsKey {
    /// Get fingerprint.
    fn fingerprint(&self, short: bool) -> String;

    /// Format user data to displayable string.
    fn user_display(&self) -> String;
}

/// Reformat the given fingerprint.
fn format_fingerprint<S: AsRef<str>>(fingerprint: S) -> String {
    fingerprint.as_ref().trim().to_uppercase()
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

#[derive(Debug, Error)]
pub enum KeychainErr {
    #[error("failed to read public key")]
    ReadKey(#[source] std::io::Error),

    #[error("failed to write public key")]
    WriteKey(#[source] std::io::Error),

    #[error("failed to import key")]
    Import(#[source] anyhow::Error),

    #[error("failed to export key")]
    Export(#[source] anyhow::Error),
}

/// Prelude traits.
pub mod prelude {
    pub use super::{ContextAdapter, Crypto, Decrypt, Encrypt, IsKey, IsKeychain};
}
