use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{Store, crypto::prelude::*};
use thiserror::Error;

#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;
use crate::util::{clipboard, error};
use crate::{
    cmd::matcher::{
        MainMatcher, Matcher,
        totp::{TotpMatcher, copy::CopyMatcher},
    },
    util::{secret, select, totp},
};

/// A TOTP copy action.
pub struct Copy<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Copy<'a> {
    /// Construct a new init action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the init action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let _matcher_totp = TotpMatcher::with(self.cmd_matches).unwrap();
        let matcher_copy = CopyMatcher::with(self.cmd_matches).unwrap();

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
            select::store_select_secret(&store, matcher_copy.query()).ok_or(Err::NoneSelected)?;

        secret::print_name(matcher_copy.query(), &secret, &store, matcher_main.quiet());

        let mut plaintext = crate::crypto::context(&matcher_main)?
            .decrypt_file(&secret.path)
            .map_err(Err::Read)?;

        // Trim plaintext to property
        if let Some(property) = matcher_copy.property() {
            plaintext = plaintext.property(property).map_err(Err::Property)?;
        }

        // Get TOTP instance, determine timeout
        let totp = totp::find_token(&plaintext)
            .ok_or(Err::NoTotp)?
            .map_err(Err::Totp)?;
        let timeout = matcher_copy
            .timeout()
            .unwrap_or(Ok(crate::CLIPBOARD_TIMEOUT))?;

        let mut copied = false;

        // Use background token recopy implementation if token changes within timeout
        let ttl = totp.ttl().map_err(Err::Totp)?;
        if timeout > ttl && !matcher_copy.no_recopy() {
            match totp::spawn_process_totp_recopy(&totp, timeout) {
                Ok(_) => {
                    if !matcher_main.quiet() {
                        eprintln!("Token copied to clipboard. Clearing after {timeout} seconds...",);
                    }
                    copied = true;
                }
                Err(err) => error::print_error(Err::Recopy(err).into()),
            }
        }

        // Fall back to simply copy
        if !copied {
            clipboard::copy_plaintext(
                totp.generate_current().map_err(Err::Totp)?,
                false,
                true,
                matcher_main.quiet(),
                matcher_main.verbose(),
                timeout,
            )?;
            if !matcher_main.quiet() {
                eprintln!("Token copied to clipboard. Clearing after {timeout} seconds...",);
            }
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

    #[error("failed to generate TOTP token")]
    Totp(#[source] anyhow::Error),

    #[error("failed to use TOTP recopy system, falling back to simple copy")]
    Recopy(#[source] anyhow::Error),
}
