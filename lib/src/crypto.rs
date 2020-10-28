use std::fs;
use std::path::Path;

use anyhow::Result;
use gpgme::{Context, EncryptFlags, Protocol};
use thiserror::Error;

use crate::{
    types::{Ciphertext, Plaintext},
    Recipients,
};

/// Protocol to use.
const PROTO: Protocol = Protocol::OpenPgp;

/// GPGME encryption flags.
const ENCRYPT_FLAGS: EncryptFlags = EncryptFlags::ALWAYS_TRUST;

/// Create GNUME context.
pub fn context() -> Result<Context> {
    Context::from_protocol(PROTO).map_err(|err| Err::Context(err).into())
}

/// Encrypt given data, write to given file.
pub fn encrypt(recipients: &Recipients, mut plaintext: Plaintext) -> Result<Ciphertext> {
    let mut ciphertext = Ciphertext::empty();
    let recipients: Vec<_> = recipients.keys().iter().map(|k| k.0.clone()).collect();
    context()?
        .encrypt_with_flags(
            &recipients,
            &mut plaintext.0,
            &mut ciphertext.0,
            ENCRYPT_FLAGS,
        )
        .map_err(Err::Encrypt)?;
    Ok(ciphertext)
}

/// Encrypt the plaintext and write it to the file.
pub fn encrypt_file(recipients: &Recipients, plaintext: Plaintext, path: &Path) -> Result<()> {
    fs::write(path, &encrypt(recipients, plaintext)?.0).map_err(|err| Err::WriteFile(err).into())
}

/// Decrypt the given ciphertext.
pub fn decrypt(mut ciphertext: Ciphertext) -> Result<Plaintext> {
    let mut plaintext = Plaintext::empty();
    context()?
        .decrypt(&mut ciphertext.0, &mut plaintext.0)
        .map_err(Err::Decrypt)?;
    Ok(plaintext)
}

/// Decrypt the file at the given path.
pub fn decrypt_file(path: &Path) -> Result<Plaintext> {
    decrypt(Ciphertext(fs::read(path).map_err(Err::ReadFile)?))
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to obtain GPGME cryptography context")]
    Context(#[source] gpgme::Error),

    #[error("failed to encrypt plaintext")]
    Encrypt(#[source] gpgme::Error),

    #[error("failed to decrypt ciphertext")]
    Decrypt(#[source] gpgme::Error),

    #[error("failed to write ciphertext to file")]
    WriteFile(#[source] std::io::Error),

    #[error("failed to read ciphertext from file")]
    ReadFile(#[source] std::io::Error),
}
