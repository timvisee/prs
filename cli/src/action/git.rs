use anyhow::Result;
use clap::ArgMatches;
use prs_lib::Store;
use thiserror::Error;

use crate::cmd::matcher::{git::GitMatcher, MainMatcher, Matcher};
use crate::util;

/// Binary name.
#[cfg(not(windows))]
pub const BIN_NAME: &str = "git";
#[cfg(windows)]
pub const BIN_NAME: &str = "git.exe";

/// Git action.
pub struct Git<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Git<'a> {
    /// Construct a new git action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the git action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_git = GitMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_git.store()).map_err(Err::Store)?;

        git(&store, matcher_git.command(), matcher_main.verbose())
    }
}

/// Invoke a git command.
// TODO: call through Command directly, possibly through lib interface
pub fn git(store: &Store, cmd: String, verbose: bool) -> Result<()> {
    util::invoke_cmd(
        format!(
            "{} -C {} {}",
            prs_lib::util::bin_path(BIN_NAME),
            store.root.display(),
            cmd
        ),
        Some(&store.root),
        verbose,
    )
    .map_err(|err| Err::Invoke(err).into())
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to invoke git command")]
    Invoke(#[source] std::io::Error),
}
