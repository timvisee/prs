//! Provides GnuPG binary context adapter.

use anyhow::Result;
use thiserror::Error;
use version_compare::Version;

use super::raw_cmd::gpg_stdout_ok;
use super::{raw, Config};
use crate::crypto::{proto, Config as CryptoConfig, IsContext, Key, Proto};
use crate::{Ciphertext, Plaintext, Recipients};

/// Binary name.
#[cfg(not(windows))]
const BIN_NAME: &str = "gpg";
#[cfg(windows)]
const BIN_NAME: &str = "gpg.exe";

/// Minimum required version.
const VERSION_MIN: &str = "2.0.0";

/// Create GnuPG binary context.
pub fn context(config: &CryptoConfig) -> Result<Context, Err> {
    let mut gpg_config = find_gpg_bin().map_err(Err::Context)?;
    gpg_config.gpg_tty = config.gpg_tty;
    gpg_config.verbose = config.verbose;
    Ok(Context::from(gpg_config))
}

/// GnuPG binary context.
pub struct Context {
    /// GPG config.
    config: Config,
}

impl Context {
    /// Construct context from GPG config.
    fn from(config: Config) -> Self {
        Self { config }
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
        raw::encrypt(&self.config, &fingerprints, plaintext)
    }

    fn decrypt(&mut self, ciphertext: Ciphertext) -> Result<Plaintext> {
        raw::decrypt(&self.config, ciphertext)
    }

    fn can_decrypt(&mut self, ciphertext: Ciphertext) -> Result<bool> {
        raw::can_decrypt(&self.config, ciphertext)
    }

    fn keys_public(&mut self) -> Result<Vec<Key>> {
        Ok(raw::public_keys(&self.config)?
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
        Ok(raw::private_keys(&self.config)?
            .into_iter()
            .map(|key| {
                Key::Gpg(proto::gpg::Key {
                    fingerprint: key.0,
                    user_ids: key.1,
                })
            })
            .collect())
    }

    fn import_key(&mut self, key: &[u8]) -> Result<()> {
        raw::import_key(&self.config, key)
    }

    fn export_key(&mut self, key: Key) -> Result<Vec<u8>> {
        raw::export_key(&self.config, &key.fingerprint(false))
    }

    fn supports_proto(&self, proto: Proto) -> bool {
        proto == Proto::Gpg
    }
}

/// Find the `gpg` binary, make GPG config.
// TODO: also try default path at /usr/bin/gpg
fn find_gpg_bin() -> Result<Config> {
    let path = which::which(BIN_NAME).map_err(Err::Unavailable)?;
    let config = Config::from(path);
    test_gpg_compat(&config)?;
    Ok(config)
}

/// Test gpg binary compatibility.
fn test_gpg_compat(config: &Config) -> Result<()> {
    // Strip stdout to just the version number
    let stdout = gpg_stdout_ok(config, ["--version"])?;
    let stdout = stdout
        .trim_start()
        .lines()
        .next()
        .and_then(|stdout| stdout.trim().strip_prefix("gpg (GnuPG) "))
        .map(|stdout| stdout.trim())
        .ok_or(Err::UnexpectedOutput)?;

    // Assert minimum version number
    let ver_min = Version::from(VERSION_MIN).unwrap();
    let ver_gpg = Version::from(stdout).unwrap();
    if ver_gpg < ver_min {
        return Err(Err::UnsupportedVersion(ver_gpg.to_string()).into());
    }

    Ok(())
}

/// GnuPG binary context error.
#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to obtain GnuPG binary cryptography context")]
    Context(#[source] anyhow::Error),

    #[error("failed to find GnuPG gpg binary")]
    Unavailable(#[source] which::Error),

    #[error("failed to communicate with GnuPG gpg binary, got unexpected output")]
    UnexpectedOutput,

    #[error("failed to use GnuPG gpg binary, unsupported version: {}", _0)]
    UnsupportedVersion(String),
}
