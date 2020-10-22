use std::io::Write;
use std::path::PathBuf;

use passr::{types::Plaintext, Recipients};

const FILE_DUMMY: &str = "/tmp/passr-dummy.gpg";

const FILE_GPG_IDS: &str = "~/.password-store/.gpg-id";

fn main() {
    let path = dummy_path();

    let plaintext: Plaintext = "blablabla".into();

    let file_gpg_ids = shellexpand::tilde(FILE_GPG_IDS);
    let recipients =
        Recipients::find_from_file(file_gpg_ids.as_ref()).expect("failed to list recipients");

    // Test encrypt & decrypt
    passr::crypto::encrypt_file(&recipients, plaintext, &path).expect("failed to encrypt");
    let plaintext = passr::crypto::decrypt_file(&path).expect("failed to decrypt");

    println!("=v=v=v=v=v=v=v=v=v=");
    std::io::stdout().write_all(&plaintext.0).unwrap();
    println!("\n=^=^=^=^=^=^=^=^=^=");
}

/// Get the path to the dummy key.
fn dummy_path() -> PathBuf {
    shellexpand::tilde(FILE_DUMMY).as_ref().into()
}
