//! Provides interface for crypto recipients.

use anyhow::Result;

use super::Key;
use crate::crypto::{self, prelude::*, util};

/// A list of recipients.
///
/// This list is used to define identities (as keys) to encrypt secrets for.
/// All keys should always use the same protocol.
///
/// In the future this may support recipients using multiple protocols.
#[derive(Clone, PartialEq)]
pub struct Recipients {
    keys: Vec<Key>,
}

impl Recipients {
    /// Construct recipients set from list of keys.
    ///
    /// # Panics
    ///
    /// Panics if keys use multiple protocols.
    pub fn from(keys: Vec<Key>) -> Self {
        assert!(keys_same_proto(&keys), "recipient keys must use same proto");
        Self { keys }
    }

    /// Get recipient keys.
    pub fn keys(&self) -> &[Key] {
        &self.keys
    }

    /// Add recipient.
    ///
    /// # Panics
    ///
    /// Panics if new key uses different protocol.
    pub fn add(&mut self, key: Key) {
        self.keys.push(key);
        assert!(
            keys_same_proto(&self.keys),
            "added recipient key uses different proto"
        );
    }

    /// Remove the given key if existent.
    pub fn remove(&mut self, key: &Key) {
        self.keys.retain(|k| k != key);
    }

    /// Remove the given keys.
    ///
    /// Keys that are not found are ignored.
    pub fn remove_all(&mut self, keys: &[Key]) {
        self.keys.retain(|k| !keys.contains(k));
    }

    /// Check whether this recipient list has the given fingerprint.
    pub fn has_fingerprint(&self, fingerprint: &str) -> bool {
        self.keys
            .iter()
            .any(|k| util::fingerprints_equal(k.fingerprint(false), fingerprint))
    }
}

/// Check whether the given recipients contain any key that we have a secret key in our keychain
/// for.
pub fn contains_own_secret_key(recipients: &Recipients) -> Result<bool> {
    let secrets = Recipients::from(crypto::context(&crypto::CONFIG)?.keys_private()?);
    Ok(recipients
        .keys()
        .iter()
        .any(|k| secrets.has_fingerprint(&k.fingerprint(false))))
}

/// Check if given keys all use same proto.
///
/// Succeeds if no key is given.
fn keys_same_proto(keys: &[Key]) -> bool {
    if keys.len() < 2 {
        true
    } else {
        let proto = keys[0].proto();
        keys[1..].iter().all(|k| k.proto() == proto)
    }
}
