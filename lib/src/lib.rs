pub mod crypto;
pub(crate) mod git;
pub mod store;
pub mod sync;
#[cfg(all(feature = "tomb", target_os = "linux"))]
pub mod systemd_bin;
#[cfg(all(feature = "tomb", target_os = "linux"))]
pub mod tomb;
#[cfg(all(feature = "tomb", target_os = "linux"))]
pub(crate) mod tomb_bin;
pub mod types;
mod vendor;

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

// Re-exports
pub use crypto::{recipients::Recipients, Key};
pub use store::{Secret, Store};
pub use types::{Ciphertext, Plaintext};

/// Default password store directory.
#[cfg(not(windows))]
pub const STORE_DEFAULT_ROOT: &str = "~/.password-store";
#[cfg(windows)]
pub const STORE_DEFAULT_ROOT: &str = "~\\.password-store";
