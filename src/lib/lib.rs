pub mod crypto;
pub mod types;

use std::fs;

const FILE_GPG_IDS: &str = "~/.password-store/.gpg-id";

/// Load the recipient fingerprints.
pub(crate) fn recipient_fingerprints() -> Vec<String> {
    let path = shellexpand::tilde(FILE_GPG_IDS);
    fs::read_to_string(path.as_ref())
        .expect("failed to read GPG ids from file")
        .lines()
        .map(|l| l.into())
        .collect()
}
