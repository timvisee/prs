use std::error::Error;
use std::fs;
use std::path::Path;

use crate::types::{Ciphertext, Plaintext};

use gpgme::{Context, EncryptFlags, Protocol};

/// Protocol to use.
const PROTO: Protocol = Protocol::OpenPgp;

/// GPGME encryption flags.
const ENCRYPT_FLAGS: EncryptFlags = EncryptFlags::ALWAYS_TRUST;

/// Encrypt given data, write to given file.
pub fn encrypt(mut plaintext: Plaintext) -> Result<Ciphertext, Box<dyn Error>> {
    let mut ctx = Context::from_protocol(PROTO)?;
    ctx.set_armor(true);

    // Load recipient keys
    // TODO: supply recipients as parameter
    let recipients = crate::recipient_fingerprints();
    let keys = if !recipients.is_empty() {
        ctx.find_keys(recipients)?
            .filter_map(|x| x.ok())
            .filter(|k| k.can_encrypt())
            .collect()
    } else {
        vec![]
    };

    let mut ciphertext = Ciphertext::empty();
    ctx.encrypt_with_flags(&keys, &mut plaintext.0, &mut ciphertext.0, ENCRYPT_FLAGS)
        .map_err(|e| format!("encrypting failed: {:?}", e))?;
    Ok(ciphertext)
}

/// Encrypt the plaintext and write it to the file.
pub fn encrypt_file(path: &Path, plaintext: Plaintext) -> Result<(), Box<dyn Error>> {
    fs::write(path, &encrypt(plaintext)?.0)?;
    Ok(())
}

/// Decrypt the given ciphertext.
pub fn decrypt(mut ciphertext: Ciphertext) -> Result<Plaintext, Box<dyn Error>> {
    let mut ctx = Context::from_protocol(PROTO)?;
    let mut plaintext = Plaintext::empty();
    ctx.decrypt(&mut ciphertext.0, &mut plaintext.0)
        .map_err(|e| format!("decrypting failed: {:?}", e))?;
    Ok(plaintext)
}

/// Decrypt the file at the given path.
pub fn decrypt_file(path: &Path) -> Result<Plaintext, Box<dyn Error>> {
    decrypt(Ciphertext(fs::read(path)?))
}
