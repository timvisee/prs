use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use gpgme::{Context, Protocol};

const PROTO: Protocol = Protocol::OpenPgp;

fn main() {
    let path = dummy_path();

    // Test encrypt & decrypt
    encrypt(&path, "blablabla".into()).expect("failed to encrypt");
    decrypt(&path).expect("failed to read gpg");
}

/// Get the path to the dummy key.
fn dummy_path() -> PathBuf {
    shellexpand::tilde("~/.password-store/dummy.gpg")
        .as_ref()
        .into()
}

/// Encrypt given data, write to given file.
fn encrypt(file: &Path, data: String) -> Result<(), Box<dyn Error>> {
    let mut ctx = Context::from_protocol(PROTO)?;
    ctx.set_armor(true);

    let recipients: Vec<&str> = vec![
        "7A72F0A555E7B77A9101C53EB8DB720BC383E172",
        // "23C12F39369310509C213C63A25620DC9AE971E7",
        // "893A3C8DA29D51AB9E3907A688AC573EC5796189",
        // "A85C79FBEFC319D593C8D89969AF6BB631DB0E35",
    ];

    let keys = if !recipients.is_empty() {
        ctx.find_keys(recipients)?
            .filter_map(|x| x.ok())
            .filter(|k| k.can_encrypt())
            .collect()
    } else {
        Vec::new()
    };

    let mut input: Vec<u8> = data.bytes().collect();
    let mut output = Vec::new();
    ctx.encrypt(&keys, &mut input, &mut output)
        .map_err(|e| format!("encrypting failed: {:?}", e))?;

    io::stdout().write_all(&output)?;
    fs::write(file, output).unwrap();

    Ok(())
}

/// Decrypt data in given file.
fn decrypt(file: &Path) -> Result<String, Box<dyn Error>> {
    let mut ctx = Context::from_protocol(PROTO)?;
    let mut input = File::open(file)?;
    let mut output = Vec::new();
    ctx.decrypt(&mut input, &mut output)
        .map_err(|e| format!("decrypting failed: {:?}", e))?;

    let output = String::from_utf8(output)?;

    println!("=v=v=v=v=v=v=v=v=v=\n{}\n=^=^=^=^=^=^=^=^=^=", output);

    Ok(output)
}
