use std::fs;
use std::path::Path;

use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::store::{Secret, Store};

use crate::cmd::matcher::{r#move::MoveMatcher, MainMatcher, Matcher};
use crate::util::{cli, error, skim, sync};

/// Move secret action.
pub struct Move<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Move<'a> {
    /// Construct a new move action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the move action.
    // TODO: re-implement error handling
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_move = MoveMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_move.store()).map_err(Err::Store)?;
        let sync = store.sync();

        sync::ensure_ready(&sync);
        sync.prepare()?;

        let secret = skim::select_secret(&store, matcher_move.query()).ok_or(Err::NoneSelected)?;

        // TODO: show secret name if not equal to query, unless quiet?

        let dest = matcher_move.destination();

        // Normalize destination path
        let path = store
            .normalize_secret_path(dest, secret.path.file_name().and_then(|p| p.to_str()), true)
            .map_err(Err::NormalizePath)?;
        let new_secret = Secret::from(&store, path.to_path_buf());

        // Check if destination already exists if not forcing
        if !matcher_main.force() && path.is_file() {
            eprintln!("A secret at '{}' already exists", path.display(),);
            if !cli::prompt_yes("Overwrite?", Some(true), &matcher_main) {
                if matcher_main.verbose() {
                    eprintln!("Move cancelled");
                }
                error::quit();
            }
        }

        #[cfg(any(unix, windows))]
        {
            // Update this (relative) alias to point to the same target after moving
            update_secret_alias_target(&store, &secret, &new_secret)?;

            // Update other aliases pointing to this, to point to new location
            update_alias_for_secret_to(&store, &secret, &new_secret);
        }

        // Move secret
        fs::rename(&secret.path, path)
            .map(|_| ())
            .map_err(Err::Move)?;

        super::remove::remove_empty_secret_dir(&secret);

        sync.finalize(format!("Move from {} to {}", secret.name, new_secret.name))?;

        if !matcher_main.quiet() {
            eprintln!("Secret moved");
        }

        Ok(())
    }
}

/// Update secret if alias for moved target.
///
/// This updates the secret if it is an alias, to update its relative alias path to point to the
/// same target when it is moved.
///
/// `secret` is the secret to update the realtive path for, `future_secret` is the future path
/// `secret` will be moved to. This does not move `secret`, it just updates its relative target
/// path for when it is moved afterwards.
///
/// If `secret` is not an alias, nothing happens.
///
/// Returns `true` if a symlink has been updated.
#[cfg(any(unix, windows))]
fn update_secret_alias_target(
    store: &Store,
    secret: &Secret,
    future_secret: &Secret,
) -> Result<bool> {
    // Do not update anything if secret is not a symlink
    if !secret.path.symlink_metadata()?.file_type().is_symlink() {
        return Ok(false);
    }

    // Find the alias target secret
    let target = fs::read_link(&secret.path).map_err(Err::UpdateAlias)?;
    let target = secret
        .path
        .parent()
        .unwrap()
        .join(target)
        .canonicalize()
        .map_err(Err::UpdateAlias)?;
    let target = Secret::from(&store, target);

    // Update alias to point to same target when moved
    update_alias(&store, &target, &secret.path, &future_secret.path)?;

    Ok(true)
}

/// Update aliases for moved secret.
///
/// Update the aliases for a secret that is moved.
///
/// The `secret` is the old secret location, the `new_secret` is the location it is moved to.
/// Aliases targetting `secret` will be updated to point to `new_secret`.
#[cfg(any(unix, windows))]
fn update_alias_for_secret_to(store: &Store, secret: &Secret, new_secret: &Secret) {
    for secret in super::remove::find_symlinks_to(&store, &secret) {
        if let Err(err) = update_alias(&store, &new_secret, &secret.path, &secret.path) {
            error::print_error(err.context(
                "failed to update path of alias that points to moved secret, ignoring...",
            ));
        }
    }
}

/// Update the path of an alias.
///
/// Updates the symlink file at `symlink` to point to the new target `src`.
///
/// # Errors
///
/// Panics if the given `symlink` path is not an existing symlink.
#[cfg(any(unix, windows))]
fn update_alias(store: &Store, src: &Secret, symlink: &Path, future_symlink: &Path) -> Result<()> {
    assert!(
        symlink.symlink_metadata()?.file_type().is_symlink(),
        "failed to update symlink, not a symlink"
    );

    // Remove existing file
    fs::remove_file(symlink)
        .map(|_| ())
        .map_err(Err::UpdateAlias)?;

    // Create new symlink
    super::alias::create_alias(store, src, future_symlink, symlink)?;

    Ok(())
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("no secret selected")]
    NoneSelected,

    #[error("failed to normalize destination path")]
    NormalizePath(#[source] anyhow::Error),

    #[error("failed to move secret file")]
    Move(#[source] std::io::Error),

    #[error("failed to update alias")]
    UpdateAlias(#[source] std::io::Error),
}
