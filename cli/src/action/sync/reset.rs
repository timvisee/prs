use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{sync::Readyness, Store};
use thiserror::Error;

#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;
use crate::{
    cmd::matcher::{
        sync::{reset::ResetMatcher, SyncMatcher},
        MainMatcher, Matcher,
    },
    util::{
        cli,
        error::{self, ErrorHints, ErrorHintsBuilder},
        sync,
    },
};

/// A sync reset action.
pub struct Reset<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Reset<'a> {
    /// Construct a new reset action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the reset action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let _matcher_sync = SyncMatcher::with(self.cmd_matches).unwrap();
        let matcher_reset = ResetMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_main.store()).map_err(Err::Store)?;
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        let mut tomb = store.tomb(
            !matcher_main.verbose(),
            matcher_main.verbose(),
            matcher_main.force(),
        );
        let sync = store.sync();

        // Prepare tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::prepare_tomb(&mut tomb, &matcher_main).map_err(Err::Tomb)?;

        // Prepare sync
        sync::ensure_ready(&sync, true);
        if !matcher_reset.no_sync() {
            sync.prepare()?;
        }

        // Ensure store is dirty, or forcing
        match sync.readyness()? {
            Readyness::Dirty => {}
            _ if matcher_main.force() => {}
            Readyness::Ready => {
                error::quit_error_msg(
                    "nothing to reset, password store is not dirty",
                    ErrorHintsBuilder::from_matcher(&matcher_main)
                        .sync(true)
                        .force(true)
                        .build()
                        .unwrap(),
                );
            }
            other => {
                error::quit_error_msg(
                    format!("unexpected sync state: {:?}", other),
                    ErrorHints::from_matcher(&matcher_main),
                );
            }
        }

        // List changed files
        if !matcher_main.quiet() {
            if let Err(err) = super::status::print_changed_files(&sync, &matcher_main) {
                error::print_error(err.context("failed to print list of changed files, ignoring"));
            }
            eprintln!();
        }

        // Confirm
        eprintln!("Password store got into a dirty state unexpectedly.");
        eprintln!("Resetting the above changes may wipe sensitive information irrecoverably.");
        if !cli::prompt_yes("Reset above changes?", Some(false), &matcher_main) {
            if matcher_main.verbose() {
                eprintln!("Reset cancelled");
            }
            error::quit();
        }

        // Reset changes
        sync.reset_hard_all().map_err(Err::Reset)?;

        // Finalize sync
        if !matcher_reset.no_sync() {
            sync.finalize("Manual sync reset")?;
        }

        // Finalize tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::finalize_tomb(&mut tomb, &matcher_main, true).map_err(Err::Tomb)?;

        if !matcher_main.quiet() {
            eprintln!("Changes reset");
        }

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

    #[error("failed to reset changes")]
    Reset(#[source] anyhow::Error),
}
