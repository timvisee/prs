use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::{Secret, Store};

use crate::cmd::matcher::{alias::AliasMatcher, MainMatcher, Matcher};
use crate::util::{cli, error, select, sync};

/// Alias secret action.
pub struct Alias<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Alias<'a> {
    /// Construct a new alias action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the alias action.
    // TODO: re-implement error handling
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_alias = AliasMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_alias.store()).map_err(Err::Store)?;
        let sync = store.sync();

        sync::ensure_ready(&sync);
        sync.prepare()?;

        let secret =
            select::store_select_secret(&store, matcher_alias.query()).ok_or(Err::NoneSelected)?;
        let dest = matcher_alias.destination();

        // TODO: show secret name if not equal to query, unless quiet?

        // Normalize dest path
        let path = store
            .normalize_secret_path(dest, secret.path.file_name().and_then(|p| p.to_str()), true)
            .map_err(Err::NormalizePath)?;
        let link_secret = Secret::from(&store, path.to_path_buf());

        // Check if destination already exists if not forcing
        if !matcher_main.force() && path.is_file() {
            eprintln!("A secret at '{}' already exists", path.display(),);
            if !cli::prompt_yes("Overwrite?", Some(true), &matcher_main) {
                if matcher_main.verbose() {
                    eprintln!("Alias cancelled");
                }
                error::quit();
            }

            // Remove existing file so we can overwrite
            fs::remove_file(&path).map_err(Err::RemoveExisting)?;
        }

        // Create alias
        create_alias(&store, &secret, &path, &path)?;

        sync.finalize(format!(
            "Alias from {} to {}",
            secret.name, link_secret.name
        ))?;

        if !matcher_main.quiet() {
            eprintln!("Secret aliased");
        }

        Ok(())
    }
}

/// Create an alias.
///
/// Create an alias (symlink) file at `place_at` for a symlink at `dst` which points to `src`.
///
/// `dst` and `place_at` are usually the same.
/// This may be different to use the correct relative symlink path for a secret at `place_at` that
/// will be moved to `dst` in the future.
pub fn create_alias(store: &Store, src: &Secret, dst: &Path, place_at: &Path) -> Result<(), Err> {
    create_symlink(secret_link_path(&store, &src, &dst)?, place_at)
}

/// Create a symlink.
///
/// Create an symlink file at `dst` which points to `src`.
fn create_symlink<P, Q>(src: P, dst: Q) -> Result<(), Err>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(src, dst).map_err(Err::Symlink)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(src, dst).map_err(Err::Symlink)
    }
}

/// Determine symlink path to use.
///
/// This function determines what path to provide when creating a symlink at `dst`, which links to
/// `src`.
fn secret_link_path(store: &Store, src: &Secret, dst: &Path) -> Result<PathBuf, Err> {
    let target = src
        .relative_path(&store.root)
        .map_err(|_| Err::UnknownRoot)?;
    let depth = path_depth(store, dst)?;

    // Build and return path
    let mut path = PathBuf::from(".");
    for _ in 0..depth {
        path = path.join("..");
    }
    Ok(path.join(target.to_path_buf()))
}

/// Find path depth in the given store.
///
/// Finds the depth (in matter of directories) of a secret path in the given store.
///
/// Returns an error if the depth could not be determined, possibly because the given file is not
/// in the given root.
///
/// Returns `0` if the given secret is in the store root.
fn path_depth(store: &Store, mut path: &Path) -> Result<u16, Err> {
    let mut depth = 0;

    while let Some(parent) = path.parent() {
        path = parent;

        if store.root == path {
            return Ok(depth);
        }

        depth += 1;
    }

    Err(Err::UnknownRoot)
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("no secret selected")]
    NoneSelected,

    #[error("failed to normalize destination path")]
    NormalizePath(#[source] anyhow::Error),

    #[error("failed to symlink secret file")]
    Symlink(#[source] std::io::Error),

    #[error("failed to remove existing file to overwrite")]
    RemoveExisting(#[source] std::io::Error),

    #[error("failed to determine secret path relative to store root")]
    UnknownRoot,
}
