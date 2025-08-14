use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{
    Store,
    sync::{Readyness, Sync},
};
use thiserror::Error;

#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;
use crate::{
    cmd::matcher::{
        MainMatcher, Matcher,
        sync::{SyncMatcher, status::StatusMatcher},
    },
    util::style::highlight,
};

/// A sync status action.
pub struct Status<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Status<'a> {
    /// Construct a new status action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the status action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let _matcher_sync = SyncMatcher::with(self.cmd_matches).unwrap();
        let _matcher_status = StatusMatcher::with(self.cmd_matches).unwrap();

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

        // Show state
        let readyness = sync.readyness()?;
        let state_msg = match readyness {
            Readyness::NoSync => "not enabled".into(),
            Readyness::Ready => "ok".into(),
            Readyness::Dirty => "dirty".into(),
            Readyness::RepoState(state) => format!("other: {state:?}"),
        };
        let is_dirty = readyness == Readyness::Dirty;
        let has_remote = readyness != Readyness::NoSync && sync.has_remote()?;
        if !matcher_main.quiet() {
            println!("Sync state: {state_msg}");
            println!(
                "Uncommitted changes: {}",
                if is_dirty { "yes" } else { "no" }
            );
            println!(
                "Remote configured: {}",
                if has_remote { "yes" } else { "no" }
            );
        }

        // List changed files if dirty or in unexpected state
        let mut show_changes = is_dirty || matches!(readyness, Readyness::RepoState(_));
        if show_changes {
            if !matcher_main.quiet() {
                eprintln!();
            }
            show_changes = print_changed_files(&sync, &matcher_main)?;
        }

        // Show hints
        if !matcher_main.quiet() {
            eprintln!();
            let bin = crate::util::bin_name();
            if readyness == Readyness::NoSync {
                eprintln!(
                    "Use '{}' to initialize sync for your password store",
                    highlight(format!("{bin} sync init"))
                );
            } else {
                if readyness == Readyness::Dirty {
                    eprintln!(
                        "Use '{}' to commit dirty changes in your password store",
                        highlight(format!("{bin} sync commit"))
                    );
                    eprintln!(
                        "Use '{}' to reset dirty changes in your password store",
                        highlight(format!("{bin} sync reset"))
                    );
                }
                if show_changes {
                    eprintln!(
                        "Use '{}' to view changed files in detail",
                        highlight(format!("{bin} git status"))
                    );
                }
                if readyness != Readyness::Ready {
                    eprintln!(
                        "Use '{}' to inspect or resolve sync repository issues using git",
                        highlight(format!("{bin} git"))
                    );
                }
                if !has_remote {
                    eprintln!(
                        "Use '{}' to configure a remote",
                        highlight(format!("{bin} sync remote <GIT_URL>"))
                    );
                }
                eprintln!(
                    "Use '{}' to sync your password store",
                    highlight(format!("{bin} sync"))
                );
            }
        }

        // Finalize tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::finalize_tomb(&mut tomb, &matcher_main, true).map_err(Err::Tomb)?;

        Ok(())
    }
}

/// Print a list of changed files.
///
/// Returns false if no file was listed.
pub(super) fn print_changed_files(sync: &Sync, matcher_main: &MainMatcher) -> Result<bool> {
    // List changed files, return early if empty
    let changed_files = sync
        .changed_files_raw(!matcher_main.verbose())
        .map_err(Err::ChangedFiles)?;
    if changed_files.is_empty() {
        return Ok(false);
    }

    if !matcher_main.quiet() {
        eprintln!("Changed files:");
    }
    changed_files.lines().for_each(|line| {
        if !matcher_main.quiet() {
            println!("- {line}")
        } else {
            println!("{line}")
        }
    });

    Ok(true)
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[cfg(all(feature = "tomb", target_os = "linux"))]
    #[error("failed to prepare password store tomb for usage")]
    Tomb(#[source] anyhow::Error),

    #[error("failed to list changed files")]
    ChangedFiles(#[source] anyhow::Error),
}
