pub mod commit;
pub mod init;
pub mod remote;
pub mod reset;
pub mod status;

use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;

use prs_lib::{
    crypto,
    sync::{Readyness, Sync as StoreSync},
    Store,
};

use crate::{
    cmd::matcher::{sync::SyncMatcher, MainMatcher, Matcher},
    util::{
        cli,
        error::{self, ErrorHintsBuilder},
        sync,
    },
};

/// Sync secrets action.
pub struct Sync<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Sync<'a> {
    /// Construct a new sync action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the sync action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_sync = SyncMatcher::with(self.cmd_matches).unwrap();

        // Subcommands
        if matcher_sync.cmd_commit().is_some() {
            return commit::Commit::new(self.cmd_matches).invoke();
        }
        if matcher_sync.cmd_init().is_some() {
            return init::Init::new(self.cmd_matches).invoke();
        }
        if matcher_sync.cmd_status().is_some() {
            return status::Status::new(self.cmd_matches).invoke();
        }
        if matcher_sync.cmd_remote().is_some() {
            return remote::Remote::new(self.cmd_matches).invoke();
        }
        if matcher_sync.cmd_reset().is_some() {
            return reset::Reset::new(self.cmd_matches).invoke();
        }

        let store = Store::open(matcher_main.store()).map_err(Err::Store)?;
        let sync = StoreSync::new(&store);
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        let mut tomb = store.tomb(
            !matcher_main.verbose(),
            matcher_main.verbose(),
            matcher_main.force(),
        );

        // Prepare tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::prepare_tomb(&mut tomb, &matcher_main).map_err(Err::Tomb)?;

        // Don't sync if not initialized or no remote, show help on how to set up
        match sync.readyness()? {
            Readyness::NoSync => {
                error::quit_error_msg(
                    "sync not configured",
                    ErrorHintsBuilder::from_matcher(&matcher_main)
                        .sync_init(true)
                        .build()
                        .unwrap(),
                );
            }
            _ if !sync.has_remote()? => {
                if !matcher_main.quiet() {
                    error::print_warning(
                        "no sync remote configured, set using: prs sync remote <GIT_URL>",
                    );
                }
            }
            _ => {}
        }

        sync::ensure_ready(&sync, matcher_sync.allow_dirty());

        // Prepare, commit, finalize
        sync.prepare()?;
        sync.finalize("Sync dirty changes")?;

        // TODO: do housekeeping?

        // TODO: assume changed for now, fetch this state from syncer
        let changed = true;

        // Were done if nothing was changed
        if !changed {
            if !matcher_main.quiet() {
                eprintln!("Everything up-to-date");
            }
            return Ok(());
        }

        // Import new keys
        let confirm_callback = |fingerprint| {
            matcher_main.force()
                || cli::prompt_yes(
                    &format!("Import recipient key {fingerprint} into keychain?"),
                    Some(true),
                    &matcher_main,
                )
        };
        crypto::store::import_missing_keys_from_store(&store, confirm_callback)
            .map_err(Err::ImportRecipients)?;

        // TODO: assert not-dirty state?

        if !matcher_main.quiet() {
            eprintln!("Sync complete");
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

    #[error("failed to import store recipients")]
    ImportRecipients(#[source] anyhow::Error),
}
