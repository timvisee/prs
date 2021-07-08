use std::path::PathBuf;

use anyhow::Result;
use chbs::{config::BasicConfig, prelude::*};
use clap::ArgMatches;
use prs_lib::{
    crypto::{self, prelude::*},
    Plaintext, Secret, Store,
};
use thiserror::Error;

use crate::cmd::matcher::{generate::GenerateMatcher, MainMatcher, Matcher};
#[cfg(feature = "clipboard")]
use crate::util::clipboard;
#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;
use crate::util::{cli, edit, error, pass, secret, stdin, sync};

/// Generate secret action.
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
        let matcher_generate = GenerateMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_generate.store()).map_err(Err::Store)?;
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

        // Normalize destination path if we will store the secret
        let dest: Option<(PathBuf, Secret)> = match matcher_generate.name() {
            Some(dest) => {
                let path = store
                    .normalize_secret_path(dest, None, true)
                    .map_err(Err::NormalizePath)?;
                let secret = Secret::from(&store, path.to_path_buf());

                Some((path, secret))
            }
            None => None,
        };

        // Prepare store sync if we will store the secret
        if dest.is_some() {
            sync::ensure_ready(&sync, matcher_generate.allow_dirty());
            if !matcher_generate.no_sync() {
                sync.prepare()?;
            }
        }

        // Generate secure password/passphrase plaintext
        let mut context = crypto::context(crypto::PROTO)?;
        let mut plaintext = generate_password(&matcher_generate);

        // Check if destination already exists, if we will stor ethe secret, ask to merge if so
        if let Some(dest) = &dest {
            if !matcher_main.force() && dest.0.is_file() {
                eprintln!("A secret at '{}' already exists", dest.0.display(),);
                if !cli::prompt_yes("Merge?", Some(true), &matcher_main) {
                    if !matcher_main.quiet() {
                        eprintln!("No secret generated");
                    }
                    error::quit();
                }

                // Append existing secret exept first line to new secret
                let existing = context
                    .decrypt_file(&dest.0)
                    .and_then(|p| p.except_first_line())
                    .map_err(Err::Read)?;
                if !existing.is_empty() {
                    plaintext.append(existing, true);
                }
            }
        }

        // Append from stdin
        if matcher_generate.stdin() {
            let extra = stdin::read_plaintext(!matcher_main.quiet())?;
            plaintext.append(extra, true);
        }

        // Edit in editor
        if matcher_generate.edit() {
            if let Some(changed) = edit::edit(&plaintext).map_err(Err::Edit)? {
                plaintext = changed;
            }
        }

        // Confirm if empty secret should be stored
        if !matcher_main.force() && plaintext.is_empty() {
            if !cli::prompt_yes(
                "Generated secret is empty. Save?",
                Some(true),
                &matcher_main,
            ) {
                error::quit();
            }
        }

        // Encrypt and write changed plaintext if we need to store
        if let Some(dest) = &dest {
            // TODO: select proper recipients (use from current file?)
            let recipients = store.recipients()?;
            context
                .encrypt_file(&recipients, plaintext.clone(), &dest.0)
                .map_err(Err::Write)?;
        }

        // Copy to clipboard
        #[cfg(feature = "clipboard")]
        if matcher_generate.copy() {
            clipboard::plaintext_copy(
                plaintext.clone(),
                true,
                !matcher_main.force(),
                !matcher_main.quiet(),
                matcher_generate.timeout()?,
            )?;
        }

        // Show in stdout
        if matcher_generate.show() {
            secret::print(plaintext).map_err(Err::Print)?;
        }

        // Finalize store sync if we saved the secret
        if let Some(dest) = &dest {
            if !matcher_generate.no_sync() {
                sync.finalize(format!("Generate secret to {}", dest.1.name))?;
            }
        }

        // Finalize tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::finalize_tomb(&mut tomb, &matcher_main, true).map_err(Err::Tomb)?;

        // Determine whehter we outputted anything to stdout/stderr
        #[allow(unused_mut)]
        let mut output_any = matcher_generate.show();
        #[cfg(feature = "clipboard")]
        {
            output_any = output_any || matcher_generate.copy();
        }

        if matcher_main.verbose() || (!output_any && !matcher_main.quiet()) {
            eprintln!("Secret created");
        }

        Ok(())
    }
}

/// Generate a random password.
///
/// This generates a secure random password/passphrase based on user configuration.
fn generate_password(matcher_generate: &GenerateMatcher) -> Plaintext {
    if matcher_generate.passphrase() {
        let mut config = BasicConfig::default();
        config.words = matcher_generate.length() as usize;
        config.to_scheme().generate().into()
    } else {
        pass::generate_password(matcher_generate.length())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[cfg(all(feature = "tomb", target_os = "linux"))]
    #[error("failed to prepare password store tomb for usage")]
    Tomb(#[source] anyhow::Error),

    #[error("failed to normalize destination path")]
    NormalizePath(#[source] anyhow::Error),

    #[error("failed to edit secret in editor")]
    Edit(#[source] anyhow::Error),

    #[error("failed to read existing secret")]
    Read(#[source] anyhow::Error),

    #[error("failed to write changed secret")]
    Write(#[source] anyhow::Error),

    #[error("failed to print secret to stdout")]
    Print(#[source] std::io::Error),
}
