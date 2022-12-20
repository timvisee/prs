use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Result};
use clap::ArgMatches;
use prs_lib::{crypto::prelude::*, Store};
use thiserror::Error;
use totp_rs::TOTP;

#[cfg(feature = "clipboard")]
use crate::util::clipboard;
#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;
use crate::{
    cmd::matcher::{
        totp::{show::ShowMatcher, TotpMatcher},
        MainMatcher, Matcher,
    },
    util::{secret, select},
};

/// Default property name for TOTP secret.
const TOTP_PROPERTY: &str = "totp";

/// A TOTP show action.
pub struct Show<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Show<'a> {
    /// Construct a new init action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the init action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_totp = TotpMatcher::with(self.cmd_matches).unwrap();
        let matcher_show = ShowMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_totp.store()).map_err(Err::Store)?;
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
            select::store_select_secret(&store, matcher_show.query()).ok_or(Err::NoneSelected)?;

        secret::print_name(matcher_show.query(), &secret, &store, matcher_main.quiet());

        let mut plaintext = crate::crypto::context(&matcher_main)?
            .decrypt_file(&secret.path)
            .map_err(Err::Read)?;

        // Trim plaintext to property
        if let Some(property) = matcher_show.property() {
            plaintext = plaintext.property(property).map_err(Err::Property)?;
        } else {
            // TODO: use this?
            plaintext = plaintext.property(TOTP_PROPERTY).map_err(Err::Property)?;

            // TODO: if not found, search plaintext for otpauth URI
        }

        // TODO: validate URL, handle unwrap errors
        let otpauth = plaintext.unsecure_to_str().unwrap();
        let totp = TOTP::<Vec<u8>>::from_url(otpauth).unwrap();
        let token = totp.generate_current().unwrap();

        // Copy to clipboard
        #[cfg(feature = "clipboard")]
        if matcher_show.copy() {
            clipboard::copy_timeout(
                token.as_bytes(),
                matcher_show
                    .timeout()
                    .unwrap_or(Ok(crate::CLIPBOARD_TIMEOUT))?,
                !matcher_main.quiet(),
            )?;
        }

        println!("{}", token);

        // Clear after timeout
        if let Some(timeout) = matcher_show.timeout() {
            let timeout = timeout?;
            let mut lines = 2;

            if matcher_main.verbose() {
                lines += 2;
                eprintln!();
                eprint!("Clearing output in {} seconds...", timeout);
            }

            thread::sleep(Duration::from_secs(timeout));
            eprint!("{}", ansi_escapes::EraseLines(lines));
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
}
