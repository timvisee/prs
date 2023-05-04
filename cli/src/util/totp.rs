#[cfg(feature = "clipboard")]
use std::io::{Error as IoError, Write};
#[cfg(feature = "clipboard")]
use std::process::{Child, Stdio};
use std::time::SystemTimeError;

use anyhow::Result;
use linkify::{LinkFinder, LinkKind};
use prs_lib::Plaintext;
use thiserror::Error;
use totp_rs::{Algorithm, Secret, TOTP};

/// OTPAUTH URL scheme.
const OTPAUTH_SCHEME: &str = "otpauth://";

/// Possible property names to search in for TOTP tokens.
const PROPERTY_NAMES: [&str; 2] = ["totp", "2fa"];

/// Try to find a TOTP token in the given plaintext.
///
/// Returns `None` if no TOTP is found.
pub fn find_token(plaintext: &Plaintext) -> Option<Result<Totp>> {
    // Find first TOTP URL globally
    match find_otpauth_url(plaintext) {
        totp @ Some(_) => return totp,
        None => {}
    }

    // Find first TOTP in common properties
    match PROPERTY_NAMES
        .iter()
        .flat_map(|p| plaintext.property(p))
        .find_map(|p| find_token(&p))
    {
        totp @ Some(_) => return totp,
        None => {}
    }

    // Try to parse full secret as encoded TOTP secret
    parse_encoded(plaintext).map(Ok)
}

/// Scan the plaintext for `otpauth` URLs.
fn find_otpauth_url(plaintext: &Plaintext) -> Option<Result<Totp>> {
    // Configure linkfinder
    let mut finder = LinkFinder::new();
    finder.url_must_have_scheme(true);
    finder.kinds(&[LinkKind::Url]);

    finder
        .links(plaintext.unsecure_to_str().unwrap())
        .filter(|l| l.as_str().starts_with(OTPAUTH_SCHEME))
        .map(|l| Totp::from_url(l.as_str()))
        .next()
}

/// Try to parse a base32 encoded TOTP token from the given plaintext.
///
/// Uses RFC6238 defaults, see:
/// - https://docs.rs/totp-rs/3.1.0/totp_rs/struct.Rfc6238.html#method.with_defaults
/// - https://tools.ietf.org/html/rfc6238
fn parse_encoded(plaintext: &Plaintext) -> Option<Totp> {
    // Trim plaintext, must be base32 encoded
    let plaintext = plaintext.unsecure_to_str().unwrap().trim();
    if !is_base32(plaintext) {
        return None;
    }

    // Encoded secret must have at least 16 bytes
    if plaintext.len() < 16 {
        return None;
    }

    // Decode to bytes
    let secret = Secret::Encoded(plaintext.to_string());
    let bytes = secret.to_bytes().unwrap();

    // Parse RFC6238 TOTP (with looser requirements)
    Some(TOTP::new_unchecked(Algorithm::SHA1, 6, 1, 30, bytes, None, "".into()).into())
}

/// Format a token as a string.
///
/// If `quiet` is `true` the token is printed with no formatting or TTL.
/// If a TTL is specified, it is printed after.
pub fn format_token(token: &Plaintext, quiet: bool, ttl: Option<u64>) -> Plaintext {
    // If quiet, print regularly
    if quiet {
        return token.clone();
    }

    // Format with spaces
    let mut formatted = if token.unsecure_ref().len() > 5 {
        Plaintext::from(
            token
                .unsecure_ref()
                .chunks(3)
                .map(|c| std::str::from_utf8(c).unwrap())
                .collect::<Vec<_>>()
                .join(" "),
        )
    } else {
        token.clone()
    };
    if let Some(ttl) = ttl {
        formatted.append(format!(" (valid for {ttl}s)").into(), false);
    }

    formatted
}

/// Print a nicely formatted token.
///
/// If `quiet` is `true` the token is printed with no formatting or TTL.
/// If a TTL is specified, it is printed after.
pub fn print_token(token: &Plaintext, quiet: bool, ttl: Option<u64>) {
    println!(
        "{}",
        format_token(token, quiet, ttl).unsecure_to_str().unwrap()
    );
}

/// A secure TOTP type.
///
/// This TOTP type outputs tokens as secure `Plaintext` and zeroes on drop.
pub struct Totp {
    totp: TOTP,
}

impl Totp {
    /// Construct a TOTP from the given TOTP URL.
    pub fn from_url(url: &str) -> Result<Self> {
        TOTP::from_url_unchecked(url)
            .map(|t| t.into())
            .map_err(|e| Err::Url(e).into())
    }

    /// Generate a token from the current system time.
    pub fn generate_current(&self) -> Result<Plaintext> {
        self.totp
            .generate_current()
            .map(|t| t.into())
            .map_err(|e| Err::Time(e).into())
    }

    /// Generate an URL for this TOTP secret.
    pub fn generate_url(&self) -> Plaintext {
        self.totp.get_url().into()
    }

    /// Give the ttl (in seconds) of the current token.
    pub fn ttl(&self) -> Result<u64> {
        self.totp.ttl().map_err(|e| Err::Time(e).into())
    }
}

impl From<TOTP> for Totp {
    fn from(totp: TOTP) -> Self {
        Self { totp }
    }
}

/// Check if string is base32 compliant
///
/// RFC: https://www.rfc-editor.org/rfc/rfc4648#page-9
pub fn is_base32(material: &str) -> bool {
    material
        .chars()
        .all(|c| c.is_ascii_uppercase() || ('2'..='7').contains(&c))
}

/// Copy the given data to the clipboard in a subprocess.
/// Revert to the old data after the given timeout.
#[cfg(feature = "clipboard")]
pub(crate) fn spawn_process_totp_recopy(totp: &Totp, timeout_sec: u64) -> Result<Child> {
    use super::{base64, cmd};

    // Spawn & disown background process to set clipboard
    let mut process = cmd::current_cmd()
        .ok_or(Err::NoSubProcess)?
        .args(["internal", "totp-recopy"])
        .arg("--timeout")
        .arg(format!("{timeout_sec}"))
        .stdin(Stdio::piped())
        .spawn()
        .map_err(Err::SpawnProcess)?;

    // Send data to copy to process
    writeln!(
        process.stdin.as_mut().unwrap(),
        "{}",
        base64::encode(totp.generate_url().unsecure_to_str().unwrap()),
    )
    .map_err(Err::ConfigProcess)?;

    Ok(process)
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("invalid TOTP secret URL")]
    Url(#[source] totp_rs::TotpUrlError),

    #[cfg(feature = "clipboard")]
    #[error("failed to use clipboard, no way to spawn subprocess for clipboard manager, must run as standalone binary")]
    NoSubProcess,

    #[cfg(feature = "clipboard")]
    #[error("failed to spawn subprocess for clipboard manager")]
    SpawnProcess(#[source] IoError),

    #[cfg(feature = "clipboard")]
    #[error("failed to configure subprocess for clipboard manager")]
    ConfigProcess(#[source] IoError),

    #[error("TOTP system time error")]
    Time(#[source] SystemTimeError),
}
