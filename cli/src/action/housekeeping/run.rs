use std::fs::{self, OpenOptions};
use std::io::{Read, Write};

use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::Store;

/// Platform specific line ending character.
#[cfg(not(windows))]
const LINE_ENDING: &'static str = "\n";
#[cfg(windows)]
const LINE_ENDING: &'static str = "\r\n";

use crate::{
    cmd::matcher::{
        housekeeping::{run::RunMatcher, HousekeepingMatcher},
        MainMatcher, Matcher,
    },
    util::sync,
};

/// A housekeeping run action.
pub struct Run<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Run<'a> {
    /// Construct a new run action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the run action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_housekeeping = HousekeepingMatcher::with(self.cmd_matches).unwrap();
        let matcher_run = RunMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_housekeeping.store()).map_err(Err::Store)?;
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        let tomb = store.tomb();

        // Prepare tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb.prepare().map_err(Err::Tomb)?;

        housekeeping(&store, matcher_run.allow_dirty(), matcher_run.no_sync())?;

        // Finalize tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb.finalize().map_err(Err::Tomb)?;

        if !matcher_main.quiet() {
            eprintln!("Housekeeping done");
        }

        Ok(())
    }
}

/// Run housekeeping tasks.
pub(crate) fn housekeeping(store: &Store, allow_dirty: bool, no_sync: bool) -> Result<()> {
    let sync = store.sync();

    // Prepare sync
    sync::ensure_ready(&sync, allow_dirty);
    if !no_sync {
        sync.prepare()?;
    }

    set_store_permissions(&store).map_err(Err::Perms)?;

    if sync.is_init() {
        set_git_ignore(&store).map_err(Err::GitAttributes)?;
        set_git_attributes(&store).map_err(Err::GitAttributes)?;
    }

    // Finalize sync
    if !no_sync {
        sync.finalize("Housekeeping")?;
    }

    Ok(())
}

/// Set the password store directory permissions to a secure default.
#[cfg(unix)]
fn set_store_permissions(store: &Store) -> Result<(), std::io::Error> {
    use std::os::unix::fs::PermissionsExt;

    // Query existing permissions, update file mode to 600
    let root = &store.root;
    let mut perms = fs::metadata(root)?.permissions();
    perms.set_mode(0o700);
    fs::set_permissions(root, perms)
}

/// Set the password store directory permissions to a secure default.
#[cfg(not(unix))]
fn set_store_permissions(_store: &Store) -> Result<(), std::io::Error> {
    // Not supported on non-Unix
    Ok(())
}

/// Set up the git ignore file.
fn set_git_ignore(store: &Store) -> Result<(), std::io::Error> {
    const ENTRIES: [&str; 5] = [".host", ".last", ".tty", ".uid", ".timer"];

    let file = store.root.join(".gitignore");

    // Create file if it doesn't exist
    if !file.is_file() {
        fs::write(&file, ENTRIES.join(LINE_ENDING))?;
        return Ok(());
    }

    // Open and read file
    let mut file = OpenOptions::new()
        .append(true)
        .read(true)
        .write(true)
        .open(file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Add each entry if it doesn't exist
    for entry in &ENTRIES {
        if !contents.lines().any(|l| &l.trim() == entry) {
            file.write_all(LINE_ENDING.as_bytes())?;
            file.write_all(entry.as_bytes())?;
        }
    }

    Ok(())
}

/// Set up the git attributes file.
fn set_git_attributes(store: &Store) -> Result<(), std::io::Error> {
    const GPG_ENTRY: &str = "*.gpg diff=gpg";

    let file = store.root.join(".gitattributes");

    // Create file if it doesn't exist
    if !file.is_file() {
        fs::write(&file, GPG_ENTRY)?;
        return Ok(());
    }

    // Open and read file
    let mut file = OpenOptions::new()
        .append(true)
        .read(true)
        .write(true)
        .open(file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Append GPG entry if it doesn't exist
    if !contents.lines().any(|l| l.trim() == GPG_ENTRY) {
        file.write_all(LINE_ENDING.as_bytes())?;
        file.write_all(GPG_ENTRY.as_bytes())?;
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[cfg(all(feature = "tomb", target_os = "linux"))]
    #[error("failed to prepare password store tomb for usage")]
    Tomb(#[source] anyhow::Error),

    #[error("failed to set password store permissions")]
    Perms(#[source] std::io::Error),

    #[error("failed to set default .gitattributes")]
    GitAttributes(#[source] std::io::Error),
}
