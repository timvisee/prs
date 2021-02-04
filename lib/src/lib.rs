pub mod crypto;
pub(crate) mod git;
pub mod store;
pub mod sync;
pub mod types;

// Re-exports
pub use crypto::{recipients::Recipients, Key};

/// Default password store directory.
pub const STORE_DEFAULT_ROOT: &str = "~/.password-store";
