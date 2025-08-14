use std::thread;
use std::time::Duration;

use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{Store, crypto::prelude::*};
use thiserror::Error;

#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;
use crate::{
    cmd::matcher::{
        MainMatcher, Matcher,
        totp::{TotpMatcher, live::LiveMatcher},
    },
    util::{
        secret, select,
        totp::{self, Totp},
    },
};

/// A TOTP live action.
pub struct Live<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Live<'a> {
    /// Construct a new init action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the init action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let _matcher_totp = TotpMatcher::with(self.cmd_matches).unwrap();
        let matcher_live = LiveMatcher::with(self.cmd_matches).unwrap();

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
            select::store_select_secret(&store, matcher_live.query()).ok_or(Err::NoneSelected)?;

        secret::print_name(matcher_live.query(), &secret, &store, matcher_main.quiet());

        let mut plaintext = crate::crypto::context(&matcher_main)?
            .decrypt_file(&secret.path)
            .map_err(Err::Read)?;

        // Trim plaintext to property
        if let Some(property) = matcher_live.property() {
            plaintext = plaintext.property(property).map_err(Err::Property)?;
        }

        // Get current TOTP token
        let totp = totp::find_token(&plaintext)
            .ok_or(Err::NoTotp)?
            .map_err(Err::Totp)?;

        // Finalize tomb before watching tokens
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::finalize_tomb(&mut tomb, &matcher_main, false).map_err(Err::Tomb)?;

        // Watch or follow tokens
        if !matcher_live.follow() {
            watch(totp, matcher_main.quiet())?;
        } else {
            follow(totp, matcher_main.quiet())?;
        }

        Ok(())
    }
}

/// Watch the token.
///
/// Show countdown if not quiet, clear when a new token is shown.
fn watch(totp: Totp, quiet: bool) -> Result<()> {
    loop {
        let token = totp.generate_current().map_err(Err::Totp)?;
        let ttl = totp.ttl().map_err(Err::Totp)?;

        totp::print_token(&token, quiet, Some(ttl));

        thread::sleep(Duration::from_secs(if !quiet { 1 } else { ttl }));
        eprint!("{}", ansi_escapes::EraseLines(2));
    }
}

/// Follow the token.
///
/// Keep printing new tokens on a new line as they arrive.
fn follow(totp: Totp, quiet: bool) -> Result<()> {
    loop {
        let token = totp.generate_current().map_err(Err::Totp)?;
        let ttl = totp.ttl().map_err(Err::Totp)?;

        totp::print_token(&token, quiet, Some(ttl));
        thread::sleep(Duration::from_secs(ttl));
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

    #[error("failed to generate TOTP token")]
    Totp(#[source] anyhow::Error),
}
