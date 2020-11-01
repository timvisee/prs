use std::fs;
use std::path::Path;

use anyhow::Result;
use gpgme::{Context, EncryptFlags, Protocol};
use thiserror::Error;
use zeroize::Zeroize;

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
pub fn encrypt(recipients: &Recipients, plaintext: Plaintext) -> Result<Ciphertext> {
    // TODO: do not use temporary (unsecure) buffer here
    let mut ciphertext = vec![];

    let recipients: Vec<_> = recipients.keys().iter().map(|k| k.0.clone()).collect();
    context()?
        .encrypt_with_flags(
            &recipients,
            plaintext.unsecure_ref(),
            &mut ciphertext,
            ENCRYPT_FLAGS,
        )
        .map_err(Err::Encrypt)?;

    // Explicit zeroing of unsecure buffer required
    let result = ciphertext.to_vec().into();
    ciphertext.zeroize();
    Ok(result)
}

/// Encrypt the plaintext and write it to the file.
pub fn encrypt_file(recipients: &Recipients, plaintext: Plaintext, path: &Path) -> Result<()> {
    fs::write(path, encrypt(recipients, plaintext)?.unsecure_ref())
        .map_err(|err| Err::WriteFile(err).into())
}

/// Decrypt the given ciphertext.
pub fn decrypt(ciphertext: Ciphertext) -> Result<Plaintext> {
    // TODO: do not use temporary (unsecure) buffer here
    let mut plaintext = vec![];

    context()?
        .decrypt(ciphertext.unsecure_ref(), &mut plaintext)
        .map_err(Err::Decrypt)?;

    // Explicit zeroing of unsecure buffer required
    let result = Ok(plaintext.to_vec().into());
    plaintext.zeroize();
    result
}

/// Decrypt the file at the given path.
pub fn decrypt_file(path: &Path) -> Result<Plaintext> {
    decrypt(fs::read(path).map_err(Err::ReadFile)?.into())
}

/// Check whether we can decrypt a file.
///
/// This checks whether we own the proper secret key to decrypt it.
///
/// To check this, actual decryption is attempted, see this if this can be improved:
/// https://stackoverflow.com/q/64633736/1000145
pub fn can_decrypt(ciphertext: Ciphertext) -> Result<bool> {
    // Try to decrypt, explicit zeroing of unsecure buffer required
    // TODO: do not use temporary (unsecure) buffer here
    let mut plaintext = vec![];
    let result = context()?.decrypt(ciphertext.unsecure_ref(), &mut plaintext);
    plaintext.zeroize();

    match result {
        Ok(_) => Ok(true),
        Err(err) if gpgme::error::Error::NO_SECKEY.code() == err.code() => Ok(false),
        _ => Ok(true),
    }
}

/// Check whether we can decrypt a file at the given path.
pub fn can_decrypt_file(path: &Path) -> Result<bool> {
    can_decrypt(fs::read(path).map_err(Err::ReadFile)?.into())
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
