use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{crypto::prelude::*, Store};
use thiserror::Error;

#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;
use crate::{
    cmd::matcher::{
        totp::{qr::QrMatcher, TotpMatcher},
        MainMatcher, Matcher,
    },
    util::{secret, select, totp},
};

/// A TOTP QR code action.
pub struct Qr<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Qr<'a> {
    /// Construct a new init action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the init action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let _matcher_totp = TotpMatcher::with(self.cmd_matches).unwrap();
        let matcher_qr = QrMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_main.store()).map_err(Err::Store)?;
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        let mut tomb = store.tomb(
            !matcher_main.verbose(),
            matcher_main.verbose(),
            matcher_main.force(),
        );

        // Prepare tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::prepare_tomb(&mut tomb, &matcher_main).map_err(Err::Tomb)?;

        let secret =
            select::store_select_secret(&store, matcher_qr.query()).ok_or(Err::NoneSelected)?;

        secret::print_name(matcher_qr.query(), &secret, &store, matcher_main.quiet());

        let mut plaintext = crate::crypto::context(&matcher_main)?
            .decrypt_file(&secret.path)
            .map_err(Err::Read)?;

        // Trim plaintext to property
        if let Some(property) = matcher_qr.property() {
            plaintext = plaintext.property(property).map_err(Err::Property)?;
        }

        // Get current TOTP token
        let totp = totp::find_token(&plaintext)
            .ok_or(Err::NoTotp)?
            .map_err(Err::Parse)?;
        let url = totp.generate_url();

        // Print TOTP URL and QR code
        if !matcher_main.quiet() {
            print!("TOTP: ");
        }
        println!("{}", url.unsecure_to_str().unwrap_or("?"));
        if !matcher_main.quiet() {
            qr2term::print_qr(url.unsecure_ref()).map_err(Err::Qr)?;
        }

        // Finalize tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::finalize_tomb(&mut tomb, &matcher_main, false).map_err(Err::Tomb)?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[cfg(all(feature = "tomb", target_os = "linux"))]
    #[error("failed to prepare password store tomb for usage")]
    Tomb(#[source] anyhow::Error),

    #[error("no secret selected")]
    NoneSelected,

    #[error("failed to read secret")]
    Read(#[source] anyhow::Error),

    #[error("failed to select property from secret")]
    Property(#[source] anyhow::Error),

    #[error("no TOTP secret found")]
    NoTotp,

    #[error("failed to parse TOTP secret")]
    Parse(#[source] anyhow::Error),

    #[error("failed to generate and print QR code")]
    Qr(#[source] qr2term::QrError),
}
