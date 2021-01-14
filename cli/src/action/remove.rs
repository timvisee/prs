use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::ArgMatches;
use thiserror::Error;
use walkdir::WalkDir;

use prs_lib::store::{Secret, SecretIterConfig, Store};

use crate::cmd::matcher::{remove::RemoveMatcher, MainMatcher, Matcher};
use crate::util::{cli, error, skim, sync};

/// Remove secret action.
pub struct Remove<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Remove<'a> {
    /// Construct a new remove action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the remove action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_remove = RemoveMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_remove.store()).map_err(Err::Store)?;
        let sync = store.sync();

        sync::ensure_ready(&sync);
        sync.prepare()?;

        let secret =
            skim::select_secret(&store, matcher_remove.query()).ok_or(Err::NoneSelected)?;

        // TODO: if this secret is a symlink, ask whether to remove target file as well?

        if !remove_confirm(&store, &secret, &matcher_main, &mut Vec::new())? {
            if matcher_main.verbose() {
                eprintln!("Removal cancelled");
            }
            error::quit();
        };

        sync.finalize(format!("Remove secret {}", secret.name))?;

        if !matcher_main.quiet() {
            eprintln!("Secret removed");
        }

        Ok(())
    }
}

/// Confirm to remove the given secret, then remove.
///
/// This also asks to remove an alias target, and aliases targeting this secret, effectively asking
/// to remove all linked aliases.
fn remove_confirm(
    store: &Store,
    secret: &Secret,
    matcher_main: &MainMatcher,
    ignore: &mut Vec<PathBuf>,
) -> Result<bool> {
    // Prevent infinite loops, skip removal if already on ignore list
    if ignore.contains(&secret.path) {
        return Ok(false);
    }

    // Check wheher secret is an alias, build prompt
    #[cfg(any(unix, windows))]
    let is_alias = fs::symlink_metadata(&secret.path)?.file_type().is_symlink();
    #[cfg(not(any(windows, unix)))]
    let is_alias = false;
    let prompt = &format!(
        "Remove {}'{}'?",
        if is_alias { "alias " } else { "" },
        secret.path.display(),
    );

    // Confirm removal
    ignore.push(secret.path.clone());
    if !matcher_main.force() && !cli::prompt_yes(&prompt, Some(true), &matcher_main) {
        return Ok(false);
    }

    // Ask to remove alias target
    if is_alias {
        match secret.alias_target(&store) {
            Ok(secret) => {
                // TODO: is this error okay?
                if let Err(err) = remove_confirm(&store, &secret, &matcher_main, ignore) {
                    error::print_error(err.context("failed to remove alias target, ignoring"));
                }
            }
            Err(err) => error::print_error(err.context("failed to query alias target, ignoring")),
        }
    }

    // Ask to remove aliases targeting this secret
    #[cfg(any(unix, windows))]
    {
        for secret in find_symlinks_to(&store, &secret) {
            if let Err(err) = remove_confirm(store, &secret, matcher_main, ignore) {
                error::print_error(err.context("failed to remove alias, ignoring"));
            }
        }
    }

    // Remove secret, remove directories that become empty
    fs::remove_file(&secret.path)
        .map(|_| ())
        .map_err(|err| Err::Remove(err))?;
    remove_empty_secret_dir(&secret);

    Ok(true)
}

/// Find symlink secrets to given secret.
///
/// Collect all secrets that are a symlink which target the given `secret`.
#[cfg(any(unix, windows))]
pub fn find_symlinks_to(store: &Store, secret: &Secret) -> Vec<Secret> {
    // Configure secret iterator to only find symlinks
    let mut config = SecretIterConfig::default();
    config.find_files = false;
    config.find_symlink_files = true;

    // Collect secrets that symlink to given secret
    store
        .secret_iter_config(config)
        .filter(|sym| {
            // Find symlink target path
            let sym_path = match std::fs::read_link(&sym.path) {
                Ok(path) => path,
                Err(_) => return false,
            };

            // Ignore secret if absolute symlink target doesn't match secret
            sym.path
                .parent()
                .unwrap()
                .join(&sym_path)
                .canonicalize()
                .map(|full_path| secret.path == full_path)
                .unwrap_or(false)
        })
        .collect()
}

/// Remove secret directory if empty.
///
/// This removes the directory the given `secret` was in if the directory is empty.
/// Parent directories will be removed if they're empty as well.
///
/// If the given `secret` still exists, the directory is never removed because it is not empty.
///
/// This never errors, but reports an error to the user when it does.
pub fn remove_empty_secret_dir(secret: &Secret) {
    // Remove secret directory if empty
    if let Err(err) = remove_empty_dir(secret.path.parent().unwrap(), true) {
        error::print_error(
            anyhow!(err).context("failed to remove now empty secret directory, ignoring"),
        );
    }
}

/// Remove directory if it's empty.
///
/// Remove the directory `path` if it's empty.
/// If the directory contains other empty directories, it's still considered empty.
///
/// If `remove_empty_parents` is true, the parents that are empty will be removed too.
fn remove_empty_dir(path: &Path, remove_empty_parents: bool) -> Result<(), io::Error> {
    // Stop if path is not an existing directory
    if !path.is_dir() {
        return Ok(());
    }

    // Make sure directory is empty, assume no on error, stop if not empty
    let is_empty = WalkDir::new(&path)
        .follow_links(true)
        .into_iter()
        .filter(|entry| match entry {
            Ok(entry) => entry.file_type().is_file(),
            Err(_) => true,
        })
        .next()
        .is_some();
    if is_empty {
        return Ok(());
    }

    // Remove the directory
    fs::remove_dir_all(path)?;

    // Remove empty parents
    if remove_empty_parents {
        if let Some(parent) = path.parent() {
            return remove_empty_dir(parent, true);
        }
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("no secret selected")]
    NoneSelected,

    #[error("failed to remove secret file")]
    Remove(#[source] std::io::Error),
}
