use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{crypto::prelude::*, Store};
use thiserror::Error;

#[cfg(feature = "clipboard")]
use crate::util::clipboard;
#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;
use crate::{
    cmd::matcher::{
        totp::{copy::CopyMatcher, TotpMatcher},
        MainMatcher, Matcher,
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
        let matcher_totp = TotpMatcher::with(self.cmd_matches).unwrap();
        let matcher_copy = CopyMatcher::with(self.cmd_matches).unwrap();

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
            select::store_select_secret(&store, matcher_copy.query()).ok_or(Err::NoneSelected)?;

        secret::print_name(matcher_copy.query(), &secret, &store, matcher_main.quiet());

        let mut plaintext = crate::crypto::context(&matcher_main)?
            .decrypt_file(&secret.path)
            .map_err(Err::Read)?;

        // Trim plaintext to property
        if let Some(property) = matcher_copy.property() {
            plaintext = plaintext.property(property).map_err(Err::Property)?;
        }

        // Get TOTP instance
        let totp = totp::find_token(&plaintext)
            .ok_or(Err::NoTotp)?
            .map_err(Err::Totp)?;

        // Determine until time based on timeout
        let timeout = matcher_copy
            .timeout()
            .unwrap_or(Ok(crate::CLIPBOARD_TIMEOUT))?;
        let until = Instant::now() + Duration::from_secs(timeout);

        // Keep recopying chaning token until the copy timeout is reached
        loop {
            // Calculate remaining timeout time, get current TOTP TTL
            let remaining_timeout = until.duration_since(std::time::Instant::now());
            let ttl = totp.ttl().map_err(Err::Totp)?;

            // Calculate current clipboard timeou
            let clip_timeout = ttl.min(remaining_timeout.as_secs() + 1);
            clipboard::plaintext_copy(
                totp.generate_current().map_err(Err::Totp)?,
                false,
                !matcher_main.force(),
                false,
                clip_timeout,
            )?;

            // We're done looping if remaining timeout is less than token TTL or we don't recopy
            let ttl_duration = Duration::from_secs(ttl);
            let done = matcher_copy.no_recopy() || remaining_timeout <= ttl_duration;

            // Report if not quiet
            if !matcher_main.quiet() {
                if done {
                    eprintln!(
                        "Token copied to clipboard. Clearing after {} seconds...",
                        clip_timeout
                    );
                } else {
                    eprintln!(
                        "Token copied to clipboard. Copying changing token after {} seconds...",
                        clip_timeout,
                    );
                }
            }

            // Break if done or wait for TTL for next loop
            if done {
                break;
            } else {
                thread::sleep(ttl_duration);
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
}
