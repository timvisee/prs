use anyhow::Result;
use gpgme::{Context as GpgmeContext, EncryptFlags, Protocol};
use thiserror::Error;
use zeroize::Zeroize;

use crate::types::{Ciphertext, Plaintext};
use crate::Recipients;

/// Protocol to use.
const PROTO: Protocol = Protocol::OpenPgp;

/// GPGME encryption flags.
const ENCRYPT_FLAGS: EncryptFlags = EncryptFlags::ALWAYS_TRUST;

/// Create GPGME crypto context.
pub fn context() -> Result<Context, Err> {
    Ok(Context::from(
        gpgme::Context::from_protocol(PROTO).map_err(|err| Err::Context(err).into())?,
    ))
}

/// GPGME crypto context.
pub struct Context {
    /// GPGME crytp context.
    context: GpgmeContext,
}

impl Context {
    pub fn from(context: GpgmeContext) -> Self {
        Self { context }
    }
}

impl super::ContextAdapter for Context {}

impl super::Crypto for Context {}

impl super::Encrypt for Context {
    fn encrypt(&mut self, recipients: &Recipients, plaintext: Plaintext) -> Result<Ciphertext> {
        // TODO: do not use temporary (unsecure) buffer here
        let mut ciphertext = vec![];

        let recipients: Vec<_> = recipients.keys().iter().map(|k| k.0.clone()).collect();
        self.context
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
}

impl super::Decrypt for Context {
    fn decrypt(&mut self, ciphertext: Ciphertext) -> Result<Plaintext> {
        // TODO: do not use temporary (unsecure) buffer here
        let mut plaintext = vec![];

        self.context
            .decrypt(ciphertext.unsecure_ref(), &mut plaintext)
            .map_err(Err::Decrypt)?;

        // Explicit zeroing of unsecure buffer required
        let result = Ok(plaintext.to_vec().into());
        plaintext.zeroize();
        result
    }

    // To check this, actual decryption is attempted, see this if this can be improved:
    // https://stackoverflow.com/q/64633736/1000145
    fn can_decrypt(&mut self, ciphertext: Ciphertext) -> Result<bool> {
        // Try to decrypt, explicit zeroing of unsecure buffer required
        // TODO: do not use temporary (unsecure) buffer here
        let mut plaintext = vec![];
        let result = self
            .context
            .decrypt(ciphertext.unsecure_ref(), &mut plaintext);
        plaintext.zeroize();

        match result {
            Ok(_) => Ok(true),
            Err(err) if gpgme::error::Error::NO_SECKEY.code() == err.code() => Ok(false),
            // TODO: should this be false for other errors?
            Err(_) => Ok(true),
        }
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to obtain GPGME cryptography context")]
    Context(#[source] gpgme::Error),

    #[error("failed to encrypt plaintext")]
    Encrypt(#[source] gpgme::Error),

    #[error("failed to decrypt ciphertext")]
    Decrypt(#[source] gpgme::Error),
}
