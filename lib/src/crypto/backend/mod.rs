//! Crypto backends.
//!
//! This module groups all crytpo backend implementations.

#[cfg(feature = "backend-gnupg-bin")]
pub mod gnupg_bin;
#[cfg(feature = "backend-gpgme")]
pub mod gpgme;
