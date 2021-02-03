use std::path::Path;

use anyhow::Result;
#[cfg(feature = "backend-gpgme")]
use gpgme::{Context, Protocol};
use thiserror::Error;

use crate::{
    crypt,
    crypt::extra::{prelude::*, Proto},
    types::{Ciphertext, Plaintext},
    Recipients,
};

/// Crypto type.
const PROTO: Proto = Proto::Gpg;

/// Protocol to use.
const GPGME_PROTO: Protocol = Protocol::OpenPgp;

/// Create GNUME context.
pub fn context() -> Result<Context> {
    Context::from_protocol(GPGME_PROTO).map_err(|err| Err::Context(err).into())
}

/// Encrypt given data, write to given file.
pub fn encrypt(recipients: &Recipients, plaintext: Plaintext) -> Result<Ciphertext> {
    crypt::extra::context(PROTO)?.encrypt(recipients, plaintext)
}

/// Encrypt the plaintext and write it to the file.
pub fn encrypt_file(recipients: &Recipients, plaintext: Plaintext, path: &Path) -> Result<()> {
    crypt::extra::context(PROTO)?.encrypt_file(recipients, plaintext, path)
}

/// Decrypt the given ciphertext.
pub fn decrypt(ciphertext: Ciphertext) -> Result<Plaintext> {
    crypt::extra::context(PROTO)?.decrypt(ciphertext)
}

/// Decrypt the file at the given path.
pub fn decrypt_file(path: &Path) -> Result<Plaintext> {
    crypt::extra::context(PROTO)?.decrypt_file(path)
}

/// Check whether we can decrypt a file.
///
/// This checks whether we own the proper secret key to decrypt it.
///
/// To check this, actual decryption is attempted, see this if this can be improved:
/// https://stackoverflow.com/q/64633736/1000145
pub fn can_decrypt(ciphertext: Ciphertext) -> Result<bool> {
    crypt::extra::context(PROTO)?.can_decrypt(ciphertext)
}

/// Check whether we can decrypt a file at the given path.
pub fn can_decrypt_file(path: &Path) -> Result<bool> {
    crypt::extra::context(PROTO)?.can_decrypt_file(path)
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to obtain GPGME cryptography context")]
    Context(#[source] gpgme::Error),
}
