use std::error::Error;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use gpgme::{Context, EncryptFlags, Protocol};

/// Protocol to use.
const PROTO: Protocol = Protocol::OpenPgp;

const FILE_DUMMY: &str = "~/.password-store/dummy.gpg";
const FILE_GPG_IDS: &str = "~/.password-store/.gpg-id";

fn main() {
    let path = dummy_path();

    // Test encrypt & decrypt
    encrypt(&path, "blablabla".into()).expect("failed to encrypt");
    decrypt(&path).expect("failed to read gpg");
}

/// Get the path to the dummy key.
fn dummy_path() -> PathBuf {
    shellexpand::tilde(FILE_DUMMY).as_ref().into()
}

/// Load the recipients to use from
fn load_recipients() -> Vec<String> {
    let path = shellexpand::tilde(FILE_GPG_IDS);
    fs::read_to_string(path.as_ref())
        .expect("failed to read GPG ids from file")
        .lines()
        .map(|l| l.into())
        .collect()
}

/// Encrypt given data, write to given file.
fn encrypt(file: &Path, data: String) -> Result<(), Box<dyn Error>> {
    let mut ctx = Context::from_protocol(PROTO)?;
    ctx.set_armor(true);

    let recipients = load_recipients();

    let keys = if !recipients.is_empty() {
        ctx.find_keys(recipients)?
            .filter_map(|x| x.ok())
            .filter(|k| k.can_encrypt())
            .collect()
    } else {
        Vec::new()
    };

    // TODO: do not encrypt with ALWAYS_TRUST, suggest to make key trusted instead
    let flags = EncryptFlags::ALWAYS_TRUST;

    let mut input: Vec<u8> = data.bytes().collect();
    let mut output = Vec::new();
    ctx.encrypt_with_flags(&keys, &mut input, &mut output, flags)
        .map_err(|e| format!("encrypting failed: {:?}", e))?;

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
