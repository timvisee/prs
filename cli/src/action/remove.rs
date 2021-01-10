use std::fs;

use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

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

        if !remove_confirm(
            &store,
            &secret,
            &matcher_main,
            &format!("Remove '{}'?", secret.path.display()),
        )? {
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
fn remove_confirm(
    store: &Store,
    secret: &Secret,
    matcher_main: &MainMatcher,
    prompt: &str,
) -> Result<bool> {
    // Confirm removal
    if !matcher_main.force() && !cli::prompt_yes(&prompt, Some(true), &matcher_main) {
        return Ok(false);
    }

    // Remove symlinks that target this secret
    for secret in find_symlinks_to(&store, &secret) {
        if let Err(err) = remove_confirm(
            store,
            &secret,
            matcher_main,
            &format!("Remove alias '{}'?", secret.path.display()),
        ) {
            error::print_error(err.context("failed to remove alias, ignoring"));
        }
    }

    // Remove secret
    fs::remove_file(&secret.path)
        .map(|_| ())
        .map_err(|err| Err::Remove(err))?;

    Ok(true)
}

/// Find symlink secrets to given secret.
///
/// Collect all secrets that are a symlink which target the given `secret`.
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

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("no secret selected")]
    NoneSelected,

    #[error("failed to remove secret file")]
    Remove(#[source] std::io::Error),
}
