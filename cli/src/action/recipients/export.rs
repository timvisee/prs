use std::fs;
use std::io::Write;

use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{crypto::prelude::*, Plaintext, Store};
use thiserror::Error;

use crate::cmd::matcher::{
    recipients::{export::ExportMatcher, RecipientsMatcher},
    MainMatcher, Matcher,
};
#[cfg(feature = "clipboard")]
use crate::util::clipboard;
use crate::util::select;
#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;

/// A recipients export action.
pub struct Export<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Export<'a> {
    /// Construct a new export action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the export action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let _matcher_recipients = RecipientsMatcher::with(self.cmd_matches).unwrap();
        let matcher_export = ExportMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_main.store()).map_err(Err::Store)?;
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        let mut tomb = store.tomb(
            !matcher_main.verbose(),
            matcher_main.verbose(),
            matcher_main.force(),
        );
        let recipients = store.recipients().map_err(Err::Load)?;

        // Prepare tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::prepare_tomb(&mut tomb, &matcher_main).map_err(Err::Tomb)?;

        let key = select::select_key(recipients.keys(), None)
            .ok_or(Err::NoneSelected)?
            .clone();

        // Export public key
        let data = Plaintext::from(crate::crypto::context(&matcher_main)?.export_key(key)?);

        let mut stdout = true;

        // Output to file
        if let Some(path) = matcher_export.output_file() {
            stdout = false;
            fs::write(path, data.unsecure_ref()).map_err(Err::Output)?;
            if !matcher_main.quiet() {
                eprintln!("Key exported to: {}", path);
            }
        }

        // Copy to clipboard
        #[cfg(feature = "clipboard")]
        if matcher_export.copy() {
            stdout = false;
            clipboard::copy(&data, matcher_main.quiet(), matcher_main.verbose())
                .map_err(Err::Clipboard)?;
        }

        if stdout {
            std::io::stdout()
                .write_all(data.unsecure_ref())
                .map_err(Err::Output)?;
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

    #[error("failed to load recipients from keychain")]
    Load(#[source] anyhow::Error),

    #[error("no key selected")]
    NoneSelected,

    #[error("failed to write key to file")]
    Output(#[source] std::io::Error),

    #[cfg(feature = "clipboard")]
    #[error("failed to copy key to clipboard")]
    Clipboard(#[source] anyhow::Error),
}
