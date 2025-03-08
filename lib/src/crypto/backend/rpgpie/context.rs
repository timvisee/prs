//! Provides GPGME binary context adapter.

use std::ops::Deref;

use anyhow::Result;
use thiserror::Error;

use crate::crypto::{Config, IsContext, Key, Proto};
use crate::{Ciphertext, Plaintext, Recipients};
use rpgpie_certificate_store::{Error as StoreError, Store};

use super::raw;

/// Create rpgpie crypto context.
pub fn context(_config: &Config) -> Result<Context, Error> {
    let store = Store::new()?;
    Ok(Context::from(store))
}

/// rpgpie crypto context.
pub struct Context {
    /// rpgpie crytp context.
    pub(super) store: rpgpie_certificate_store::Store,
}

impl Context {
    pub fn from(store: Store) -> Self {
        Self { store }
    }
}

impl IsContext for Context {
    fn encrypt(&mut self, recipients: &Recipients, plaintext: Plaintext) -> Result<Ciphertext> {
        let fps = recipients
            .keys()
            .iter()
            .map(|k| k.fingerprint(false))
            .collect::<Vec<_>>();
        raw::encrypt(&mut *self, fps.deref(), plaintext.unsecure_ref())
    }

    fn decrypt(&mut self, ciphertext: Ciphertext) -> Result<Plaintext> {
        Ok(raw::decrypt(&mut *self, ciphertext.unsecure_ref())?)
    }

    fn can_decrypt(&mut self, ciphertext: Ciphertext) -> Result<bool> {
        let res = raw::decrypt(&mut *self, ciphertext.unsecure_ref());
        match res {
            Ok(_) => Ok(true),
            Err(Error::NoSecretKey) => Ok(false),
            Err(err) => Err(err.into()),
        }
    }

    fn keys_public(&mut self) -> Result<Vec<Key>> {
        let certs = self.store.search_like_user_id("%")?;

        Ok(certs
            .into_iter()
            .map(|c| raw::metadata_for_cert(&c))
            .collect())
    }

    fn keys_private(&mut self) -> Result<Vec<Key>> {
        // TODO: Enumerate a configurable cert store for private key material
        let store = rpgpie_certificate_store::Store::new()?;

        // This is called rarely and we need accurate results
        let mut keys = Vec::new();
        let cards = raw::cards()?;

        for mut card in cards {
            let fingerprint = card
                .transaction()?
                .fingerprint(openpgp_card::ocard::KeyType::Decryption)?;

            if let Some(fp) = fingerprint {
                let certs = store.search_by_fingerprint(&fp.to_hex())?;
                for cert in certs {
                    keys.push(raw::metadata_for_cert(&cert));
                }
            }
        }

        Ok(keys)
    }

    fn import_key(&mut self, key: &[u8]) -> Result<()> {
        let certs = rpgpie::certificate::Certificate::load(&mut std::io::Cursor::new(key))?;
        for cert in certs {
            let existing = self
                .store
                .get_by_primary_fingerprint(&raw::format_fingerprint(cert.fingerprint()))?;

            if existing.is_none() {
                self.store.insert(&cert)?;
            }
        }

        Ok(())
    }

    fn export_key(&mut self, key: Key) -> Result<Vec<u8>> {
        let mut certs = self
            .store
            .get_by_primary_fingerprint(&key.fingerprint(false))?
            .into_iter()
            .chain(self.store.search_by_fingerprint(&key.fingerprint(false))?);

        if let Some(cert) = certs.next() {
            let mut data = Vec::new();
            cert.save(true, &mut data)?;
            Ok(data)
        } else {
            Err(Error::CertificateMissing.into())
        }
    }

    fn supports_proto(&self, proto: Proto) -> bool {
        proto == Proto::Gpg
    }
}

/// rpgpie context error.
#[derive(Debug, Error)]
pub enum Error {
    /// Error for accessing the rpgpie certificate store
    #[error("Certificate store: {0:?}")]
    CertificateStore(#[from] StoreError),
    #[error("Smartcard error: {0}")]
    Smartcard(#[from] openpgp_card::Error),
    #[error("PGP error: {0}")]
    Pgp(#[from] pgp::errors::Error),
    #[error("No secret key to decrypt message")]
    NoSecretKey,
    #[error("No public key to encrypt message")]
    NoPublicKey,
    #[error("Unimplemented ({0})")]
    Unimplemented(String),
    #[error("Malformed OpenPGP message data")]
    MalformedMessage,
    #[error("No usable public keys")]
    NoUsablePublicKeys,
    #[error("Certificate missing from key store")]
    CertificateMissing,
    #[error("{0}")]
    Any(#[from] anyhow::Error),
}
