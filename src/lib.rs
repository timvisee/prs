use std::error::Error;
use std::fs;
use std::path::Path;

use gpgme::{Context, EncryptFlags, Protocol};

/// Protocol to use.
const PROTO: Protocol = Protocol::OpenPgp;

const FILE_GPG_IDS: &str = "~/.password-store/.gpg-id";

const ENCRYPT_FLAGS: EncryptFlags = EncryptFlags::ALWAYS_TRUST;

/// Ciphertext.
pub struct Ciphertext(pub Vec<u8>);

impl Ciphertext {
    /// New empty ciphertext.
    pub fn empty() -> Self {
        Self(vec![])
    }
}

/// Plaintext.
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
}

impl From<&str> for Plaintext {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().into())
    }
}

/// Load the recipient fingerprints.
pub(crate) fn recipient_fingerprints() -> Vec<String> {
    let path = shellexpand::tilde(FILE_GPG_IDS);
    fs::read_to_string(path.as_ref())
        .expect("failed to read GPG ids from file")
        .lines()
        .map(|l| l.into())
        .collect()
}

/// Encrypt given data, write to given file.
pub fn encrypt(mut plaintext: Plaintext) -> Result<Ciphertext, Box<dyn Error>> {
    let mut ctx = Context::from_protocol(PROTO)?;
    ctx.set_armor(true);

    // Load recipient keys
    // TODO: supply recipients as parameter
    let recipients = recipient_fingerprints();
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
    fs::write(path, encrypt(plaintext)?.0)?;
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
