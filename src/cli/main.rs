use std::io::Write;
use std::path::PathBuf;

use passr::types::Plaintext;

const FILE_DUMMY: &str = "/tmp/passr-dummy.gpg";

fn main() {
    let path = dummy_path();

    let plaintext: Plaintext = "blablabla".into();

    // Test encrypt & decrypt
    passr::crypto::encrypt_file(&path, plaintext).expect("failed to encrypt");
    let plaintext = passr::crypto::decrypt_file(&path).expect("failed to decrypt");

    println!("=v=v=v=v=v=v=v=v=v=");
    std::io::stdout().write_all(&plaintext.0).unwrap();
    println!("\n=^=^=^=^=^=^=^=^=^=");
}

/// Get the path to the dummy key.
fn dummy_path() -> PathBuf {
    shellexpand::tilde(FILE_DUMMY).as_ref().into()
}
