use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::{store::Store, Recipients};

use crate::cmd::matcher::{
    recipients::{generate::GenerateMatcher, RecipientsMatcher},
    MainMatcher, Matcher,
};
use crate::util::{
    self, cli,
    error::{self, ErrorHintsBuilder},
    style, sync,
};

/// A recipients generate action.
pub struct Generate<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Generate<'a> {
    /// Construct a new generate action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the generate action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_recipients = RecipientsMatcher::with(self.cmd_matches).unwrap();
        let matcher_generate = GenerateMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_recipients.store()).map_err(Err::Store)?;
        let sync = store.sync();

        sync::ensure_ready(&sync);
        sync.prepare()?;

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
        let new = gpg_generate(matcher_main.quiet(), matcher_main.verbose())?;
        let new_keys = new.keys();

        if !matcher_generate.no_add() {
            if new.keys().is_empty() {
                error::quit_error_msg(
                    "not adding recipient to store because no new keys are found",
                    ErrorHintsBuilder::default()
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
                    crate::action::housekeeping::recrypt::recrypt_all(
                        &store,
                        matcher_main.quiet(),
                        matcher_main.verbose(),
                    )
                    .map_err(Err::Recrypt)?;
                };
            } else {
                if !matcher_main.quiet() {
                    super::add::cannot_decrypt_show_recrypt_hints();
                }
            }

            sync.finalize(format!(
                "Generate and add recipient {}",
                new_keys
                    .into_iter()
                    .map(|k| k.fingerprint(true))
                    .collect::<Vec<_>>()
                    .join(", "),
            ))?;

            if !matcher_main.quiet() {
                for key in new_keys {
                    eprintln!("Added recipient: {}", key);
                }
            }
        }

        Ok(())
    }
}

/// Invoke GPG generate command.
///
/// Return new keys as recipients.
pub fn gpg_generate(quiet: bool, verbose: bool) -> Result<Recipients> {
    // List recipients before
    let before = prs_lib::all(true)?;

    // Generate key through GPG
    if !quiet {
        eprintln!("===== GPG START =====");
    }
    util::invoke_cmd("gpg --full-generate-key".into(), None, verbose).map_err(Err::Invoke)?;
    if !quiet {
        eprintln!("===== GPG END =====");
    }

    // List recipients after, keep new keys
    let mut diff = prs_lib::all(true)?;
    diff.remove_many(before.keys());
    Ok(diff)
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to load recipients from keychain")]
    Load(#[source] anyhow::Error),

    #[error("failed to invoke gpg command")]
    Invoke(#[source] std::io::Error),

    #[error("failed to re-encrypt secrets in store")]
    Recrypt(#[source] anyhow::Error),
}
