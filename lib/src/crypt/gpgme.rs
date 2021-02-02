use std::fmt;

use anyhow::Result;
use gpgme::{Context as GpgmeContext, EncryptFlags, Protocol};
use thiserror::Error;
use zeroize::Zeroize;

use super::prelude::*;
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

    pub fn inner_mut(&mut self) -> &mut GpgmeContext {
        &mut self.context
    }
}

impl ContextAdapter for Context {}

impl Crypto for Context {
    fn keychain<'a>(&'a mut self) -> Box<dyn IsKeychain + 'a> {
        Box::new(Keychain::from(self))
    }
}

impl Encrypt for Context {
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

impl Decrypt for Context {
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

/// GPGME key, a recipient.
#[derive(Clone)]
pub struct Key(gpgme::Key);

impl IsKey for Key {
    /// Get fingerprint.
    fn fingerprint(&self, short: bool) -> String {
        let fp = self.0.fingerprint().expect("key does not have fingerprint");
        super::format_fingerprint(if short { &fp[fp.len() - 16..] } else { fp })
    }

    /// Format user data to displayable string.
    fn user_display(&self) -> String {
        self.0
            .user_ids()
            .map(|user| {
                let mut parts = vec![];
                if let Ok(name) = user.name() {
                    if !name.trim().is_empty() {
                        parts.push(name.into());
                    }
                }
                if let Ok(comment) = user.comment() {
                    if !comment.trim().is_empty() {
                        parts.push(format!("({})", comment));
                    }
                }
                if let Ok(email) = user.email() {
                    if !email.trim().is_empty() {
                        parts.push(format!("<{}>", email));
                    }
                }
                parts.join(" ")
            })
            .collect::<Vec<_>>()
            .join("; ")
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.0.id_raw() == other.0.id_raw() && self.0.fingerprint_raw() == other.0.fingerprint_raw()
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[GPG] {} - {}",
            self.fingerprint(true),
            self.user_display()
        )
    }
}

/// Provides access to GPGME keys.
pub struct Keychain<'a> {
    context: &'a mut GpgmeContext,
}

impl<'a> Keychain<'a> {
    fn from(context: &'a mut Context) -> Self {
        Self {
            context: context.inner_mut(),
        }
    }
}

impl<'a> IsKeychain for Keychain<'a> {
    fn keys_public(&mut self) -> Result<Vec<Box<dyn IsKey>>> {
        Ok(self
            .context
            .keys()?
            .into_iter()
            .filter_map(|k| k.ok())
            .filter(|k| k.can_encrypt())
            .map(|k| Box::new(Key(k)) as Box<dyn IsKey>)
            .collect())
    }

    fn keys_private(&mut self) -> Result<Vec<Box<dyn IsKey>>> {
        Ok(self
            .context
            .secret_keys()?
            .into_iter()
            .filter_map(|k| k.ok())
            .filter(|k| k.can_encrypt())
            .map(|k| Box::new(Key(k)) as Box<dyn IsKey>)
            .collect())
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
