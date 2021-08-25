//! Crypto interface.
//!
//! This module provides an interface to all cryptography features that are used in prs.
//!
//! It supports multiple cryptography protocols (e.g. GPG) and multiple backends (e.g. GPGME,
//! GnuPG). The list of supported protocols and backends may be extended in the future.

pub mod backend;
pub mod proto;
pub mod recipients;
pub mod store;
pub mod util;

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;

use anyhow::Result;
use thiserror::Error;

use crate::{Ciphertext, Plaintext, Recipients};

/// Crypto protocol.
///
/// This list contains all protocols supported by the prs project. This does not mean that all
/// protocols are supported at runtime in a given build.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Proto {
    /// GPG crypto.
    Gpg,
}

impl Proto {
    /// Get the protocol display name.
    pub fn name(&self) -> &str {
        match self {
            Self::Gpg => "GPG",
        }
    }
}

/// Crypto configuration.
///
/// Allows configuring extra properties for contexts globally.
pub struct Config {
    /// Protocol used.
    pub proto: Proto,

    /// Use TTY for password input with GPG.
    pub gpg_tty: bool,
}

impl Config {
    /// Construct config with given protocol.
    pub fn from(proto: Proto) -> Self {
        Self {
            proto,
            gpg_tty: false,
        }
    }
}

/// Represents a key.
///
/// The key type may be any of the supported crypto proto types.
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

    /// Display string for user.
    pub fn display(&self) -> String {
        match self {
            #[cfg(feature = "_crypto-gpg")]
            Key::Gpg(key) => key.display_user(),
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} - {}",
            self.proto().name(),
            self.fingerprint(true),
            self.display(),
        )
    }
}

/// Get crypto context for given proto type at runtime.
///
/// This selects a compatible crypto context at runtime.
///
/// # Errors
///
/// Errors if no compatible crypto context is available for the selected protocol because no
/// backend is providing it. Also errors if creating the context fails.
pub fn context(config: &Config) -> Result<Context, Err> {
    // Select proper crypto backend
    match config.proto {
        Proto::Gpg => {
            #[cfg(feature = "backend-gpgme")]
            return Ok(Context::from(Box::new(
                backend::gpgme::context::context(config).map_err(|err| Err::Context(err.into()))?,
            )));
            #[cfg(feature = "backend-gnupg-bin")]
            return Ok(Context::from(Box::new(
                backend::gnupg_bin::context::context(config)
                    .map_err(|err| Err::Context(err.into()))?,
            )));
        }
    }

    #[allow(unreachable_code)]
    Err(Err::Unsupported(config.proto))
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

    fn supports_proto(&self, proto: Proto) -> bool {
        self.context.supports_proto(proto)
    }
}

/// Defines generic crypto context.
///
/// Implemented on backend specific cryptography contexcts, makes using it possible through a
/// single simple interface.
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

    /// Obtain a public key from keychain for fingerprint.
    fn get_public_key(&mut self, fingerprint: &str) -> Result<Key> {
        self.keys_public()?
            .into_iter()
            .find(|key| util::fingerprints_equal(key.fingerprint(false), fingerprint))
            .ok_or_else(|| Err::UnknownFingerprint.into())
    }

    /// Find public keys from keychain for fingerprints.
    ///
    /// Skips fingerprints no key is found for.
    fn find_public_keys(&mut self, fingerprints: &[&str]) -> Result<Vec<Key>> {
        let keys = self.keys_public()?;
        Ok(fingerprints
            .into_iter()
            .filter_map(|fingerprint| {
                keys.iter()
                    .find(|key| util::fingerprints_equal(key.fingerprint(false), fingerprint))
                    .cloned()
            })
            .collect())
    }

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

    /// Check whether this context supports the given protocol.
    fn supports_proto(&self, proto: Proto) -> bool;
}

/// A pool of proto contexts.
///
/// Makes using multiple contexts easy, by caching contexts by protocol type and initializing them
/// on demand.
pub struct ContextPool {
    /// All loaded contexts.
    contexts: HashMap<Proto, Context>,
}

impl ContextPool {
    /// Create new empty pool.
    pub fn empty() -> Self {
        Self {
            contexts: HashMap::new(),
        }
    }

    /// Get mutable context for given proto.
    ///
    /// This will initialize the context if no context is loaded for the given proto yet. This
    /// may error..
    pub fn get_mut<'a>(&'a mut self, config: &'a Config) -> Result<&'a mut Context> {
        Ok(self
            .contexts
            .entry(config.proto)
            .or_insert(context(config)?))
    }
}

/// Crypto error.
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

    #[error("fingerprint does not match public key in keychain")]
    UnknownFingerprint,
}

/// Prelude for common crypto traits.
pub mod prelude {
    pub use super::{store::StoreRecipients, IsContext};
}
