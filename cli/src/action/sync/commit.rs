use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{sync::Readyness, Store};
use thiserror::Error;

#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;
use crate::{
    cmd::matcher::{
        sync::{commit::CommitMatcher, SyncMatcher},
        MainMatcher, Matcher,
    },
    util::{
        cli,
        error::{self, ErrorHints, ErrorHintsBuilder},
        sync,
    },
};

/// A sync commit action.
pub struct Commit<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Commit<'a> {
    /// Construct a new commit action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the commit action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let _matcher_sync = SyncMatcher::with(self.cmd_matches).unwrap();
        let matcher_commit = CommitMatcher::with(self.cmd_matches).unwrap();

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
        if !matcher_commit.no_sync() {
            sync.prepare()?;
        }

        // Ensure store is dirty, or forcing
        match sync.readyness()? {
            Readyness::Dirty => {}
            _ if matcher_main.force() => {}
            Readyness::Ready => {
                error::quit_error_msg(
                    "nothing to commit, password store is not dirty",
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

        // Confirm
        eprintln!("Password store got into a dirty state unexpectedly.");
        eprintln!("Committing may break your password store and may cause unexpected results.");
        if !cli::prompt_yes("Continue to commit?", Some(false), &matcher_main) {
            if matcher_main.verbose() {
                eprintln!("Commit cancelled");
            }
            error::quit();
        }

        // Commit changes
        sync.commit_all("Manual sync commit", matcher_main.force())
            .map_err(Err::Commit)?;

        // Finalize sync
        if !matcher_commit.no_sync() {
            sync.finalize("Manual sync commit")?;
        }

        // Finalize tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::finalize_tomb(&mut tomb, &matcher_main, true).map_err(Err::Tomb)?;

        if !matcher_main.quiet() {
            eprintln!("Changes committed");
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

    #[error("failed to commit changes")]
    Commit(#[source] anyhow::Error),
}
