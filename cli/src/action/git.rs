use anyhow::Result;
use clap::ArgMatches;
use prs_lib::Store;
use prs_lib::sync::{Readyness, Sync as StoreSync};
use thiserror::Error;

use crate::cmd::matcher::{MainMatcher, Matcher, git::GitMatcher};
use crate::util;
#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;

/// Binary name.
#[cfg(not(windows))]
pub const BIN_NAME: &str = "git";
#[cfg(windows)]
pub const BIN_NAME: &str = "git.exe";

/// Git action.
pub struct Git<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Git<'a> {
    /// Construct a new git action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the git action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_git = GitMatcher::with(self.cmd_matches).unwrap();

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

        // Warn if sync is not configured
        if sync
            .readyness()
            .map(|r| r == Readyness::NoSync)
            .unwrap_or(false)
        {
            util::error::print_warning("sync not configured, store is not a git repository");
        }

        #[cfg_attr(
            not(all(feature = "tomb", target_os = "linux")),
            expect(clippy::let_and_return)
        )]
        let result = git(&store, matcher_git.command(), matcher_main.verbose());

        // Finalize tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::finalize_tomb(&mut tomb, &matcher_main, false).map_err(Err::Tomb)?;

        result
    }
}

/// Invoke a git command.
// TODO: call through Command directly, possibly through lib interface
pub fn git(store: &Store, cmd: String, verbose: bool) -> Result<()> {
    util::invoke_cmd(
        &format!("{} -C {} {}", BIN_NAME, store.root.display(), cmd),
        Some(&store.root),
        verbose,
    )
    .map_err(|err| Err::Invoke(err).into())
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[cfg(all(feature = "tomb", target_os = "linux"))]
    #[error("failed to prepare password store tomb for usage")]
    Tomb(#[source] anyhow::Error),

    #[error("failed to invoke git command")]
    Invoke(#[source] std::io::Error),
}
