use std::time::SystemTimeError;

use anyhow::Result;
use linkify::{LinkFinder, LinkKind};
use prs_lib::Plaintext;
use thiserror::Error;
use totp_rs::{Rfc6238, Secret, TOTP};
use zeroize::Zeroize;

/// OTPAUTH URL scheme.
const OTPAUTH_SCHEME: &str = "otpauth://";

/// Possible property names to search in for TOTP tokens.
const PROPERTY_NAMES: [&str; 2] = ["totp", "2fa"];

/// Try to find a TOTP token in the given plaintext.
///
/// Returns `None` if no TOTP is found.
pub fn find_token(plaintext: &Plaintext) -> Option<Result<ZeroingTotp>> {
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
    parse_encoded(plaintext)
}

/// Scan the plaintext for `otpauth` URLs.
fn find_otpauth_url(plaintext: &Plaintext) -> Option<Result<ZeroingTotp>> {
    // Configure linkfinder
    let mut finder = LinkFinder::new();
    finder.url_must_have_scheme(true);
    finder.kinds(&[LinkKind::Url]);

    finder
        .links(plaintext.unsecure_to_str().unwrap())
        .filter(|l| l.as_str().starts_with(OTPAUTH_SCHEME))
        .map(|l| {
            TOTP::from_url(l.as_str())
                .map(|t| t.into())
                .map_err(|e| Err::Url(e).into())
        })
        .next()
}

/// Try to parse a base32 encoded TOTP token from the given plaintext.
///
/// Uses RFC6238 defaults, see:
/// - https://docs.rs/totp-rs/3.1.0/totp_rs/struct.Rfc6238.html#method.with_defaults
/// - https://tools.ietf.org/html/rfc6238
fn parse_encoded(plaintext: &Plaintext) -> Option<Result<ZeroingTotp>> {
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
    zero_secret(secret);

    // Parse RFC6238 TOTP
    Some(
        Rfc6238::with_defaults(bytes)
            .map_err(Err::Rfc6238)
            .and_then(|rfc| TOTP::from_rfc6238(rfc).map_err(Err::Url))
            .map(|t| t.into())
            .map_err(|e| e.into()),
    )
}

/// Print a nicely formatted token.
///
/// If `quiet` is `true` the token is printed with no formatting or TTL.
/// If a TTL is specified, it is printed after.
pub fn print_token(token: &Plaintext, quiet: bool, ttl: Option<u64>) {
    // If quiet, print regularly
    if quiet {
        println!("{}", token.unsecure_to_str().unwrap());
        return;
    }

    // Format with spaces
    let formatted = if token.unsecure_ref().len() > 5 {
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
        println!(
            "{} (valid for {}s)",
            formatted.unsecure_to_str().unwrap(),
            ttl
        );
    } else {
        println!("{}", formatted.unsecure_to_str().unwrap());
    }
}

/// Securely zero the `Secret` type.
fn zero_secret(secret: Secret) {
    match secret {
        Secret::Encoded(mut encoded) => encoded.zeroize(),
        Secret::Raw(mut raw) => raw.zeroize(),
    }
}

/// A secure TOTP type that zeroes on drop.
pub struct ZeroingTotp {
    totp: TOTP,
}

impl ZeroingTotp {
    /// Generate a token from the current system time.
    pub fn generate_current(&self) -> Result<Plaintext> {
        self.totp
            .generate_current()
            .map(|t| t.into())
            .map_err(|e| Err::Time(e).into())
    }

    /// Give the ttl (in seconds) of the current token.
    pub fn ttl(&self) -> Result<u64> {
        self.totp.ttl().map_err(|e| Err::Time(e).into())
    }
}

impl From<TOTP> for ZeroingTotp {
    fn from(totp: TOTP) -> Self {
        Self { totp }
    }
}

impl Drop for ZeroingTotp {
    fn drop(&mut self) {
        self.zeroize()
    }
}

impl Zeroize for ZeroingTotp {
    fn zeroize(&mut self) {
        let TOTP {
            ref mut digits,
            ref mut secret,
            ref mut issuer,
            ref mut account_name,
            ..
        } = self.totp;
        digits.zeroize();
        secret.zeroize();
        issuer.zeroize();
        account_name.zeroize();
    }
}

/// Check if string is base32 compliant
///
/// RFC: https://www.rfc-editor.org/rfc/rfc4648#page-9
pub fn is_base32(material: &str) -> bool {
    material
        .chars()
        .all(|c| ('A'..='Z').contains(&c) || ('2'..='7').contains(&c))
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("invalid RFC6238 TOTP secret")]
    Rfc6238(#[source] totp_rs::Rfc6238Error),

    #[error("invalid TOTP secret URL")]
    Url(#[source] totp_rs::TotpUrlError),

    #[error("TOTP system time error")]
    Time(#[source] SystemTimeError),
}
