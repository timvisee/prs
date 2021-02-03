pub mod backend;
pub mod proto;
pub mod recipients;

use std::fs;
use std::path::Path;

use anyhow::Result;
use thiserror::Error;

use crate::types::{Ciphertext, Plaintext};
use crate::Recipients;

/// Crypto protocol.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Proto {
    /// GPG crypto.
    #[cfg(feature = "_crypto-gpg")]
    Gpg,
}

/// Represents a key.
#[derive(Clone, PartialEq)]
#[non_exhaustive]
pub enum Key {
    /// An GPG key.
    #[cfg(feature = "_crypto-gpg")]
    Gpg(proto::gpg::Key),
}

impl Key {
    /// Get key protocol type.
    pub fn proto(&self) -> Proto {
        match self {
            #[cfg(feature = "_crypto-gpg")]
            Key::Gpg(_) => Proto::Gpg,
        }
    }

    /// Key fingerprint.
    pub fn fingerprint(&self, short: bool) -> String {
        match self {
            #[cfg(feature = "_crypto-gpg")]
            Key::Gpg(key) => key.fingerprint(short),
        }
    }

    /// Key displayable user data.
    pub fn display_user(&self) -> String {
        match self {
            #[cfg(feature = "_crypto-gpg")]
            Key::Gpg(key) => key.display_user(),
        }
    }
}

/// Get crypto context for given crypto type at runtime.
///
/// This selects a compatibel crypto context at runtime.
///
/// # Errors
///
/// Errors if no compatible crypto context is available because no backend is providing it. Also
/// errors if creating the context fails.
pub fn context(proto: Proto) -> Result<Context, Err> {
    // Select proper crypto backend
    match proto {
        #[allow(unreachable_code)]
        #[cfg(feature = "_crypto-gpg")]
        Proto::Gpg => {
            #[cfg(feature = "backend-gpgme")]
            return Ok(Context::from(Box::new(
                backend::gpgme::context::context().map_err(|err| Err::Context(err.into()))?,
            )));
            #[cfg(feature = "backend-gnupg-bin")]
            return Ok(Context::from(Box::new(
                backend::gnupg_bin::context::context().map_err(|err| Err::Context(err.into()))?,
            )));
        }
    }

    Err(Err::Unsupported(proto))
}

/// Generic context.
pub struct Context {
    /// Inner context.
    context: Box<dyn IsContext>,
}

impl Context {
    pub fn from(context: Box<dyn IsContext>) -> Self {
        Self { context }
    }
}

impl IsContext for Context {
    fn encrypt(&mut self, recipients: &Recipients, plaintext: Plaintext) -> Result<Ciphertext> {
        self.context.encrypt(recipients, plaintext)
    }

    fn decrypt(&mut self, ciphertext: Ciphertext) -> Result<Plaintext> {
        self.context.decrypt(ciphertext)
    }

    fn can_decrypt(&mut self, ciphertext: Ciphertext) -> Result<bool> {
        self.context.can_decrypt(ciphertext)
    }

    fn keys_public(&mut self) -> Result<Vec<Key>> {
        self.context.keys_public()
    }

    fn keys_private(&mut self) -> Result<Vec<Key>> {
        self.context.keys_private()
    }

    fn import_key(&mut self, key: &[u8]) -> Result<()> {
        self.context.import_key(key)
    }

    fn export_key(&mut self, key: Key) -> Result<Vec<u8>> {
        self.context.export_key(key)
    }
}

pub trait IsContext {
    /// Encrypt plaintext for recipients.
    fn encrypt(&mut self, recipients: &Recipients, plaintext: Plaintext) -> Result<Ciphertext>;

    /// Encrypt plaintext and write it to the file.
    fn encrypt_file(
        &mut self,
        recipients: &Recipients,
        plaintext: Plaintext,
        path: &Path,
    ) -> Result<()> {
        fs::write(path, self.encrypt(recipients, plaintext)?.unsecure_ref())
            .map_err(|err| Err::WriteFile(err).into())
    }

    /// Decrypt ciphertext.
    fn decrypt(&mut self, ciphertext: Ciphertext) -> Result<Plaintext>;

    /// Decrypt ciphertext from file.
    fn decrypt_file(&mut self, path: &Path) -> Result<Plaintext> {
        self.decrypt(fs::read(path).map_err(Err::ReadFile)?.into())
    }

    /// Check whether we can decrypt ciphertext.
    fn can_decrypt(&mut self, ciphertext: Ciphertext) -> Result<bool>;

    /// Check whether we can decrypt ciphertext from file.
    fn can_decrypt_file(&mut self, path: &Path) -> Result<bool> {
        self.can_decrypt(fs::read(path).map_err(Err::ReadFile)?.into())
    }

    /// Obtain all public keys from keychain.
    fn keys_public(&mut self) -> Result<Vec<Key>>;

    /// Obtain all public keys from keychain.
    fn keys_private(&mut self) -> Result<Vec<Key>>;

    /// Import the given key from bytes into keychain.
    fn import_key(&mut self, key: &[u8]) -> Result<()>;

    /// Import the given key from a file into keychain.
    fn import_key_file(&mut self, path: &Path) -> Result<()> {
        self.import_key(&fs::read(path).map_err(Err::ReadFile)?)
    }

    /// Export the given key from the keychain as bytes.
    fn export_key(&mut self, key: Key) -> Result<Vec<u8>>;

    /// Export the given key from the keychain to a file.
    fn export_key_file(&mut self, key: Key, path: &Path) -> Result<()> {
        fs::write(path, self.export_key(key)?).map_err(|err| Err::WriteFile(err).into())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to obtain GPGME cryptography context")]
    Context(#[source] anyhow::Error),

    #[error("failed to built context, protocol not supportd: {:?}", _0)]
    Unsupported(Proto),

    #[error("failed to write to file")]
    WriteFile(#[source] std::io::Error),

    #[error("failed to read from file")]
    ReadFile(#[source] std::io::Error),
}

pub mod prelude {
    pub use super::IsContext;
}
