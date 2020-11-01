use std::fs::{self, OpenOptions};
use std::io::{Read, Write};

use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::store::Store;

use crate::{
    cmd::matcher::{housekeeping::HousekeepingMatcher, MainMatcher, Matcher},
    util::sync,
};

/// A housekeeping run action.
pub struct Run<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Run<'a> {
    /// Construct a new run action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the run action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_housekeeping = HousekeepingMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_housekeeping.store()).map_err(Err::Store)?;

        housekeeping(&store)?;

        if matcher_main.verbose() {
            eprintln!("Housekeeping done");
        }

        Ok(())
    }
}

/// Run housekeeping tasks.
pub(crate) fn housekeeping(store: &Store) -> Result<()> {
    let sync = store.sync();

    sync::ensure_ready(&sync);
    sync.prepare()?;

    set_store_permissions(&store).map_err(Err::Perms)?;

    if sync.is_init() {
        set_git_attributes(&store).map_err(Err::GitAttributes)?;
    }

    sync.finalize("Housekeeping")
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
fn set_store_permissions(store: &Store) -> Result<(), std::io::Error> {
    // Not supported on non-Unix
    Ok(())
}

/// Set git attributes file.
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
        eprintln!("B");
        file.write_all("\n".as_bytes())?;
        eprintln!("C");
        file.write_all(GPG_ENTRY.as_bytes())?;
        eprintln!("D");
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to set password store permissions")]
    Perms(#[source] std::io::Error),

    #[error("failed to set default .gitattributes")]
    GitAttributes(#[source] std::io::Error),
}
