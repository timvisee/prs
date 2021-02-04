//! Provides GPGME binary context adapter.

use anyhow::Result;
use gpgme::{Context as GpgmeContext, Protocol};
use thiserror::Error;

use super::raw;
use crate::crypto::{proto, IsContext, Key, Proto};
use crate::{Ciphertext, Plaintext, Recipients};

/// Protocol to use.
const PROTO: Protocol = Protocol::OpenPgp;

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

impl IsContext for Context {
    fn encrypt(&mut self, recipients: &Recipients, plaintext: Plaintext) -> Result<Ciphertext> {
        let fingerprints: Vec<String> = recipients
            .keys()
            .iter()
            .map(|key| key.fingerprint(false))
            .collect();
        let fingerprints: Vec<&str> = fingerprints.iter().map(|fp| fp.as_str()).collect();
        raw::encrypt(&mut self.context, &fingerprints, plaintext)
    }

    fn decrypt(&mut self, ciphertext: Ciphertext) -> Result<Plaintext> {
        raw::decrypt(&mut self.context, ciphertext)
    }

    fn can_decrypt(&mut self, ciphertext: Ciphertext) -> Result<bool> {
        raw::can_decrypt(&mut self.context, ciphertext)
    }

    fn keys_public(&mut self) -> Result<Vec<Key>> {
        Ok(raw::public_keys(&mut self.context)?
            .into_iter()
            .map(|key| {
                Key::Gpg(proto::gpg::Key {
                    fingerprint: key.0,
                    user_ids: key.1,
                })
            })
            .collect())
    }

    fn keys_private(&mut self) -> Result<Vec<Key>> {
        Ok(raw::private_keys(&mut self.context)?
            .into_iter()
            .map(|key| {
                Key::Gpg(proto::gpg::Key {
                    fingerprint: key.0,
                    user_ids: key.1,
                })
            })
            .collect())
    }

    // TODO: implement: get_public_key
    // TODO: implement: find_public_keys

    fn import_key(&mut self, key: &[u8]) -> Result<()> {
        raw::import_key(&mut self.context, key)
    }

    fn export_key(&mut self, key: Key) -> Result<Vec<u8>> {
        raw::export_key(&mut self.context, &key.fingerprint(false))
    }

    fn supports_proto(&self, proto: Proto) -> bool {
        proto == Proto::Gpg
    }
}

/// GPGME context error.
#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to obtain GPGME cryptography context")]
    Context(#[source] gpgme::Error),
}
