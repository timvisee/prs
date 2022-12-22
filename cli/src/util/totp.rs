use linkify::{LinkFinder, LinkKind};
use prs_lib::Plaintext;
use totp_rs::{Rfc6238, Secret, TOTP};
use zeroize::Zeroize;

/// OTPAUTH URL scheme.
const OTPAUTH_SCHEME: &str = "otpauth://";

/// Possible property names to search in for TOTP tokens.
const PROPERTY_NAMES: [&str; 2] = ["2fa", "totp"];

/// Try to find a TOTOP token in the given plaintext.
///
/// Returns `None` if no TOTP is found.
pub fn find_token(plaintext: &Plaintext) -> Option<TOTP> {
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
// TODO: return result
fn find_otpauth_url(plaintext: &Plaintext) -> Option<TOTP> {
    // Configure linkfinder
    let mut finder = LinkFinder::new();
    finder.url_must_have_scheme(true);
    finder.kinds(&[LinkKind::Url]);

    finder
        .links(plaintext.unsecure_to_str().unwrap())
        .filter(|l| l.as_str().starts_with(OTPAUTH_SCHEME))
        .map(|l| {
            // TODO: don't unwrap but return error
            TOTP::<Vec<u8>>::from_url(l.as_str()).unwrap()
        })
        .next()
}

/// Try to parse a base32 encoded TOTP token from the given plaintext.
///
/// Uses RFC6238 defaults, see:
/// - https://docs.rs/totp-rs/3.1.0/totp_rs/struct.Rfc6238.html#method.with_defaults
/// - https://tools.ietf.org/html/rfc6238
fn parse_encoded(plaintext: &Plaintext) -> Option<TOTP> {
    // Trim plaintext, must be base32 encoded
    let plaintext = plaintext.unsecure_to_str().unwrap().trim();
    if !is_base32(plaintext) {
        return None;
    }

    // Decode to bytes
    let secret = Secret::Encoded(plaintext.to_string());
    let mut bytes = secret.to_bytes().unwrap();
    zero_secret(secret);

    // Secret must have at least 16 bytes
    if bytes.len() < 16 {
        bytes.zeroize();
        return None;
    }

    // TODO: do not unwrap
    let rfc = Rfc6238::with_defaults(bytes).unwrap();
    let totp = TOTP::from_rfc6238(rfc).unwrap();

    Some(totp)
}

/// Print a nicely formatted token.
///
/// If `quiet` is `true` the token is printed with no formatting.
pub fn print_token(token: &Plaintext, quiet: bool) {
    // If quiet, print regularly
    if quiet {
        println!("{}", token.unsecure_to_str().unwrap());
        return;
    }

    // Format with spaces
    let formatted = Plaintext::from(
        token
            .unsecure_ref()
            .chunks(3)
            .map(|c| std::str::from_utf8(c).unwrap())
            .collect::<Vec<_>>()
            .join(" "),
    );
    println!("{}", formatted.unsecure_to_str().unwrap());
}

/// Securely zero the `TOTP` type.
pub fn zero_totp(totp: TOTP) {
    let TOTP {
        mut digits,
        mut secret,
        mut issuer,
        mut account_name,
        ..
    } = totp;
    digits.zeroize();
    secret.zeroize();
    issuer.zeroize();
    account_name.zeroize();
}

/// Securely zero the `Secret` type.
pub fn zero_secret(secret: Secret) {
    match secret {
        Secret::Encoded(mut encoded) => encoded.zeroize(),
        Secret::Raw(mut raw) => raw.zeroize(),
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
