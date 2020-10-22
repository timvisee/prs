use std::error::Error;
use std::fs;
use std::path::Path;

use crate::{
    types::{Ciphertext, Plaintext},
    Recipients,
};

use gpgme::{Context, EncryptFlags, Protocol};

/// Protocol to use.
const PROTO: Protocol = Protocol::OpenPgp;

/// GPGME encryption flags.
const ENCRYPT_FLAGS: EncryptFlags = EncryptFlags::ALWAYS_TRUST;

/// Create GNUME context.
pub fn context() -> Result<Context, Box<dyn Error>> {
    let mut ctx = Context::from_protocol(PROTO)?;
    ctx.set_armor(true);
    Ok(ctx)
}

/// Encrypt given data, write to given file.
pub fn encrypt(
    recipients: &Recipients,
    mut plaintext: Plaintext,
) -> Result<Ciphertext, Box<dyn Error>> {
    let mut ciphertext = Ciphertext::empty();
    context()?
        .encrypt_with_flags(
            recipients.keys(),
            &mut plaintext.0,
            &mut ciphertext.0,
            ENCRYPT_FLAGS,
        )
        .map_err(|e| format!("encrypting failed: {:?}", e))?;
    Ok(ciphertext)
}

/// Encrypt the plaintext and write it to the file.
pub fn encrypt_file(
    recipients: &Recipients,
    plaintext: Plaintext,
    path: &Path,
) -> Result<(), Box<dyn Error>> {
    fs::write(path, &encrypt(recipients, plaintext)?.0)?;
    Ok(())
}

/// Decrypt the given ciphertext.
pub fn decrypt(mut ciphertext: Ciphertext) -> Result<Plaintext, Box<dyn Error>> {
    let mut plaintext = Plaintext::empty();
    context()?
        .decrypt(&mut ciphertext.0, &mut plaintext.0)
        .map_err(|e| format!("decrypting failed: {:?}", e))?;
    Ok(plaintext)
}

/// Decrypt the file at the given path.
pub fn decrypt_file(path: &Path) -> Result<Plaintext, Box<dyn Error>> {
    decrypt(Ciphertext(fs::read(path)?))
}
