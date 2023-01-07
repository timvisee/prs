use anyhow::Result;
use clap::ArgMatches;
use prs_lib::Store;
use thiserror::Error;

#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;
use crate::{
    cmd::matcher::{
        sync::{remote::RemoteMatcher, SyncMatcher},
        MainMatcher, Matcher,
    },
    util::{
        self,
        error::{self, ErrorHintsBuilder},
        style,
    },
};

/// A sync remote action.
pub struct Remote<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Remote<'a> {
    /// Construct a new remote action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the remote action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let _matcher_sync = SyncMatcher::with(self.cmd_matches).unwrap();
        let matcher_remote = RemoteMatcher::with(self.cmd_matches).unwrap();

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

        if !sync.is_init() {
            error::quit_error_msg(
                "sync is not configured",
                ErrorHintsBuilder::default()
                    .sync_init(true)
                    .build()
                    .unwrap(),
            );
        }

        // Get or set remote
        let remotes = sync.remotes()?;
        match matcher_remote.git_url() {
            Some(url) => {
                match remotes.len() {
                    0 => sync.add_remote_url("origin", url)?,
                    1 => sync.set_remote_url(&remotes[0], url)?,
                    _ => error::quit_error_msg(
                        "multiple remotes configured, cannot set automatically",
                        ErrorHintsBuilder::default().git(true).build().unwrap(),
                    ),
                }
                if !matcher_main.quiet() {
                    eprintln!("To sync with the remote now use:");
                    eprintln!(
                        "    {}",
                        style::highlight(&format!("{} sync", util::bin_name()))
                    );
                    eprintln!();
                }
                if matcher_main.verbose() {
                    eprintln!("Sync remote set");
                }
            }
            None => match remotes.len() {
                0 => error::quit_error_msg(
                    "no remote configured",
                    ErrorHintsBuilder::default()
                        .sync_remote(true)
                        .build()
                        .unwrap(),
                ),
                1 => println!("{}", sync.remote_url(&remotes[0])?),
                _ => error::quit_error_msg(
                    "multiple remotes configured, cannot decide automatically",
                    ErrorHintsBuilder::default().git(true).build().unwrap(),
                ),
            },
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
}
