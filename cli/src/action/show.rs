use std::time::Duration;

use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{Store, crypto::prelude::*};
use thiserror::Error;

use crate::cmd::matcher::{MainMatcher, Matcher, show::ShowMatcher};
#[cfg(feature = "clipboard")]
use crate::util::clipboard;
#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;
use crate::util::{secret, select};
use crate::viewer;

/// Show secret action.
pub struct Show<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Show<'a> {
    /// Construct a new show action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the show action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
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

        let secret = select::store_select_secret(&store, matcher_show.query(), &matcher_main)
            .ok_or(Err::NoneSelected)?;

        let mut plaintext = crate::crypto::context(&matcher_main)?
            .decrypt_file(&secret.path)
            .map_err(Err::Read)?;

        // Trim plaintext to first line or property
        if matcher_show.first_line() {
            plaintext = plaintext.first_line()?;
        } else if let Some(property) = matcher_show.property() {
            plaintext = plaintext.property(property).map_err(Err::Property)?;
        }

        // Copy to clipboard
        #[cfg(feature = "clipboard")]
        if matcher_show.copy() {
            clipboard::copy_plaintext(
                plaintext.clone(),
                true,
                !matcher_main.force(),
                matcher_main.quiet(),
                matcher_main.verbose(),
                matcher_show
                    .timeout()
                    .unwrap_or(Ok(crate::CLIPBOARD_TIMEOUT))?,
            )?;
        }

        // Show directly or in viewer
        if matcher_show.viewer() {
            viewer::viewer(
                &store,
                &secret,
                plaintext,
                matcher_show.timeout().transpose()?.map(Duration::from_secs),
                &matcher_main,
                matcher_show.query(),
            )
            .map_err(Err::Viewer)?;
        } else {
            secret::print_name(matcher_show.query(), &secret, &store, matcher_main.quiet());
            secret::print(plaintext).map_err(Err::Print)?
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

    #[error("failed to print secret to stdout")]
    Print(#[source] std::io::Error),

    #[error("failed to start secret viewer")]
    Viewer(#[source] anyhow::Error),
}
