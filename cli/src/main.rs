use std::io::Write;
use std::path::PathBuf;

use passr::{store::Store, types::Plaintext};

const STORE_DEFAULT_ROOT: &str = "~/.password-store";
const FILE_DUMMY: &str = "/tmp/passr-dummy.gpg";

fn main() {
    let path = dummy_path();

    let plaintext: Plaintext = "blablabla".into();

    // Open password store, get recipients
    let store = Store::open(STORE_DEFAULT_ROOT);
    let recipients = store.recipients().expect("failed to list recipients");

    // Test encrypt & decrypt
    passr::crypto::encrypt_file(&recipients, plaintext, &path).expect("failed to encrypt");
    let plaintext = passr::crypto::decrypt_file(&path).expect("failed to decrypt");

    println!("=v=v=v=v=v=v=v=v=v=");
    std::io::stdout().write_all(&plaintext.0).unwrap();
    println!("\n=^=^=^=^=^=^=^=^=^=");

    let entries = store.entries();
    for entry in entries {
        println!("{}", entry.name());
    }
}

/// Get the path to the dummy key.
fn dummy_path() -> PathBuf {
    shellexpand::tilde(FILE_DUMMY).as_ref().into()
}
