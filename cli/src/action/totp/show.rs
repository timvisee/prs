use std::time::Duration;

use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{crypto::prelude::*, Store};
use thiserror::Error;

use crate::action;
#[cfg(feature = "clipboard")]
use crate::util::clipboard;
#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;
use crate::{
    cmd::matcher::{
        totp::{show::ShowMatcher, TotpMatcher},
        MainMatcher, Matcher,
    },
    util::{secret, select, totp},
};

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
        let _matcher_totp = TotpMatcher::with(self.cmd_matches).unwrap();
        let matcher_show = ShowMatcher::with(self.cmd_matches).unwrap();

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
            select::store_select_secret(&store, matcher_show.query()).ok_or(Err::NoneSelected)?;

        let mut plaintext = crate::crypto::context(&matcher_main)?
            .decrypt_file(&secret.path)
            .map_err(Err::Read)?;

        // Trim plaintext to property
        if let Some(property) = matcher_show.property() {
            plaintext = plaintext.property(property).map_err(Err::Property)?;
        }

        // Get current TOTP token
        let totp = totp::find_token(&plaintext)
            .ok_or(Err::NoTotp)?
            .map_err(Err::Parse)?;
        let token = totp.generate_current().map_err(Err::Totp)?;
        let ttl = totp.ttl().map_err(Err::Totp)?;

        // Copy to clipboard
        #[cfg(feature = "clipboard")]
        if matcher_show.copy() {
            clipboard::plaintext_copy(
                token.clone(),
                true,
                !matcher_main.force(),
                !matcher_main.quiet(),
                matcher_show
                    .timeout()
                    .unwrap_or(Ok(crate::CLIPBOARD_TIMEOUT))?,
            )?;
        }

        // Show directly or with timeout
        match matcher_show.timeout() {
            None => {
                secret::print_name(matcher_show.query(), &secret, &store, matcher_main.quiet());
                totp::print_token(&token, matcher_main.quiet(), Some(ttl));
            }
            Some(sec) => action::show::show_timeout(
                &store,
                &secret,
                totp::format_token(&token, matcher_main.quiet(), Some(ttl)),
                Duration::from_secs(sec?),
                &matcher_main,
                matcher_show.query(),
            )
            .map_err(Err::ShowTimeout)?,
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

    #[error("failed to generate TOTP token")]
    Totp(#[source] anyhow::Error),

    #[error("failed to show secret in viewer with timeout")]
    ShowTimeout(#[source] anyhow::Error),
}
