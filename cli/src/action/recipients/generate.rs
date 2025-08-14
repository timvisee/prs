use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{Recipients, Store, crypto::prelude::*};
use thiserror::Error;

use crate::cmd::matcher::{
    MainMatcher, Matcher,
    recipients::{RecipientsMatcher, generate::GenerateMatcher},
};
#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;
use crate::util::{
    self, cli,
    error::{self, ErrorHintsBuilder},
    style, sync,
};

/// Binary name.
#[cfg(not(windows))]
const BIN_NAME: &str = "gpg";
#[cfg(windows)]
const BIN_NAME: &str = "gpg.exe";

/// A recipients generate action.
pub struct Generate<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Generate<'a> {
    /// Construct a new generate action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the generate action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let _matcher_recipients = RecipientsMatcher::with(self.cmd_matches).unwrap();
        let matcher_generate = GenerateMatcher::with(self.cmd_matches).unwrap();

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

        // Prepare sync
        sync::ensure_ready(&sync, matcher_generate.allow_dirty());
        if !matcher_generate.no_sync() {
            sync.prepare()?;
        }

        // Generating recipient in no-interact mode is not supported
        if matcher_main.no_interact() && !matcher_main.force() {
            error::quit_error_msg(
                "generating recipient with --no-interact is not supported",
                ErrorHintsBuilder::from_matcher(&matcher_main)
                    .force(true)
                    .build()
                    .unwrap(),
            )
        }

        // Show warning to user
        if !matcher_main.force() {
            eprintln!("This will start a key pair generation wizard through 'gpg'");
            if !cli::prompt_yes("Continue?", Some(true), &matcher_main) {
                if matcher_main.verbose() {
                    eprintln!("Generation cancelled");
                }
                error::quit();
            }
        }

        // Generate new key through GPG
        let new = gpg_generate(&matcher_main)?;
        let new_keys = new.keys();

        if !matcher_generate.no_add() {
            if new.keys().is_empty() {
                error::quit_error_msg(
                    "not adding recipient to store because no new keys are found",
                    ErrorHintsBuilder::from_matcher(&matcher_main)
                        .add_info(format!(
                            "Use '{}' to add a recipient",
                            style::highlight("prs recipients add")
                        ))
                        .build()
                        .unwrap(),
                );
            }

            // Add new keys to store
            let mut recipients = store.recipients().map_err(Err::Load)?;
            for key in new_keys {
                recipients.add(key.clone());
            }
            recipients.save(&store)?;

            if prs_lib::store::can_decrypt(&store) {
                // Recrypt secrets
                // TODO: do not quit on error, finish sync, ask to revert instead?
                if !matcher_generate.no_recrypt() {
                    crate::action::housekeeping::recrypt::recrypt_all(&store, &matcher_main)
                        .map_err(Err::Recrypt)?;
                };
            } else if !matcher_main.quiet() {
                super::add::cannot_decrypt_show_recrypt_hints();
            }

            // Finalize sync
            if !matcher_generate.no_sync() {
                sync.finalize(format!(
                    "Generate and add recipient {}",
                    new_keys
                        .iter()
                        .map(|k| k.fingerprint(true))
                        .collect::<Vec<_>>()
                        .join(", "),
                ))?;
            }

            if !matcher_main.quiet() {
                for key in new_keys {
                    eprintln!("Added recipient: {key}");
                }
            }
        }

        // Finalize tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::finalize_tomb(&mut tomb, &matcher_main, true).map_err(Err::Tomb)?;

        Ok(())
    }
}

/// Invoke GPG generate command.
///
/// Return new keys as recipients.
pub fn gpg_generate(matcher_main: &MainMatcher) -> Result<Recipients> {
    // List recipients before
    let mut context = crate::crypto::context(matcher_main)?;
    let before = Recipients::from(context.keys_private()?);

    // Generate key through GPG
    if !matcher_main.quiet() {
        eprintln!("===== GPG START =====");
    }
    util::invoke_cmd(
        &format!("{BIN_NAME} --full-generate-key"),
        None,
        matcher_main.verbose(),
    )
    .map_err(Err::Invoke)?;
    if !matcher_main.quiet() {
        eprintln!("===== GPG END =====");
    }

    // List recipients after, keep new keys
    let mut diff = Recipients::from(context.keys_private()?);
    diff.remove_all(before.keys());
    Ok(diff)
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[cfg(all(feature = "tomb", target_os = "linux"))]
    #[error("failed to prepare password store tomb for usage")]
    Tomb(#[source] anyhow::Error),

    #[error("failed to load recipients from keychain")]
    Load(#[source] anyhow::Error),

    #[error("failed to invoke gpg command")]
    Invoke(#[source] std::io::Error),

    #[error("failed to re-encrypt secrets in store")]
    Recrypt(#[source] anyhow::Error),
}
