use std::fs;
use std::path::Path;

use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{crypto, Store};
use thiserror::Error;

use crate::cmd::matcher::{clone::CloneMatcher, MainMatcher, Matcher};
use crate::util::{
    self,
    error::{self, ErrorHints},
    style,
};

/// Clone store action.
pub struct Clone<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Clone<'a> {
    /// Construct a new clone action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the clone action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_clone = CloneMatcher::with(self.cmd_matches).unwrap();

        let path = matcher_clone.store();
        let path = shellexpand::full(&path).map_err(Err::ExpandPath)?;

        ensure_dir_free(&Path::new(path.as_ref()))?;

        // Create store dir, open it and clone
        fs::create_dir_all(path.as_ref()).map_err(Err::Init)?;
        let store = Store::open(path.as_ref()).map_err(Err::Store)?;
        let sync = store.sync();
        sync.clone(matcher_clone.git_url(), matcher_main.quiet())
            .map_err(Err::Clone)?;

        // Import repo recipients missing in keychain
        crypto::store::import_missing_keys_from_store(&store).map_err(Err::ImportRecipients)?;

        // Run housekeeping
        crate::action::housekeeping::run::housekeeping(&store).map_err(Err::Housekeeping)?;

        // Check whether the store has any key we own the secret for, default to false
        let store_has_our_secret = store
            .recipients()
            .and_then(|recipients| crypto::recipients::contains_own_secret_key(&recipients))
            .unwrap_or(false);

        // Hint user to add our recipient key
        if !matcher_main.quiet() {
            if !store_has_our_secret {
                let bin = util::bin_name();
                let system_has_secret =
                    crypto::util::has_private_key(crypto::PROTO).unwrap_or(true);

                if system_has_secret {
                    println!("Now add your own key as recipient or generate a new one:");
                } else {
                    println!("Now generate and add a new recipient key for yourself:");
                }
                if system_has_secret {
                    println!(
                        "    {}",
                        style::highlight(&format!("{} recipients add --secret", bin))
                    );
                }
                println!(
                    "    {}",
                    style::highlight(&format!("{} recipients generate", bin))
                );
                println!();
            } else {
                eprintln!("Store cloned");
            }
        }

        Ok(())
    }
}

/// Ensure the given path is a free directory.
///
/// Checks whether the given path is not a directory, or whehter the directory is empty.
/// Quits on error.
// TODO: duplicate in action/init, move to shared module
fn ensure_dir_free(path: &Path) -> Result<()> {
    // Fine if not a directory
    if !path.is_dir() {
        return Ok(());
    }

    // Fine if no paths in dir
    if path.read_dir().map_err(Err::Init)?.count() == 0 {
        return Ok(());
    }

    error::quit_error_msg(
        format!(
            "cannot clone store, directory already exists: {}",
            path.display(),
        ),
        ErrorHints::default(),
    )
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to expand store path")]
    ExpandPath(#[source] shellexpand::LookupError<std::env::VarError>),

    #[error("failed to initialize store")]
    Init(#[source] std::io::Error),

    #[error("failed to access initialized password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to clone remote store")]
    Clone(#[source] anyhow::Error),

    #[error("failed to import store recipients")]
    ImportRecipients(#[source] anyhow::Error),

    #[error("failed to run housekeeping tasks")]
    Housekeeping(#[source] anyhow::Error),
}
