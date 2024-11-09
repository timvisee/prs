use std::path::PathBuf;

use anyhow::Result;
use chbs::{config::BasicConfig, prelude::*};
use clap::ArgMatches;
use prs_lib::{crypto::prelude::*, Plaintext, Secret, Store};
use thiserror::Error;

use crate::cmd::matcher::{generate::GenerateMatcher, MainMatcher, Matcher};
#[cfg(feature = "clipboard")]
use crate::util::clipboard;
#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;
use crate::util::{cli, edit, error, pass, secret, select, stdin, sync};

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

        // Select existing secret if merging, select based on path normally
        let dest: Option<(PathBuf, Secret)> = if matcher_generate.merge() {
            // Prepare store sync
            sync::ensure_ready(&sync, matcher_generate.allow_dirty());
            if !matcher_generate.no_sync() {
                sync.prepare()?;
            }

            // Select secret
            let secret =
                select::store_select_secret(&store, matcher_generate.name().map(|s| s.to_owned()))
                    .ok_or(Err::NoneSelected)?;

            Some((secret.path.clone(), secret))
        } else {
            match matcher_generate.name() {
                Some(dest) => {
                    let path = store
                        .normalize_secret_path(dest, None, true)
                        .map_err(Err::NormalizePath)?;
                    let secret = Secret::from(&store, path.clone());

                    // Prepare store sync
                    sync::ensure_ready(&sync, matcher_generate.allow_dirty());
                    if !matcher_generate.no_sync() {
                        sync.prepare()?;
                    }

                    Some((path, secret))
                }
                None => None,
            }
        };

        // Generate secure password/passphrase plaintext
        let mut context = crate::crypto::context(&matcher_main)?;
        let mut plaintext = generate_password(&matcher_generate);

        // If destination already exists, merge
        if let Some(dest) = &dest {
            // Ask whether to merge
            let exists = dest.0.is_file();
            if !matcher_main.force() && !matcher_generate.merge() && exists {
                eprintln!("A secret at '{}' already exists", dest.0.display(),);
                if !cli::prompt_yes("Merge?", Some(true), &matcher_main) {
                    if !matcher_main.quiet() {
                        eprintln!("No secret generated");
                    }
                    error::quit();
                }
            }

            // Append existing secret except first line to new secret
            if exists {
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
            // Quietly copy generated password to clipboard before editing
            #[cfg(feature = "clipboard")]
            if matcher_generate.copy() {
                clipboard::copy_plaintext(
                    plaintext.clone(),
                    true,
                    !matcher_main.force(),
                    !matcher_main.verbose(),
                    matcher_main.verbose(),
                    matcher_generate.timeout()?,
                )?;
            }

            if let Some(changed) = edit::edit(&plaintext).map_err(Err::Edit)? {
                plaintext = changed;
            }
        }

        // Confirm if empty secret should be stored
        if !matcher_main.force()
            && plaintext.is_empty()
            && !cli::prompt_yes(
                "Generated secret is empty. Save?",
                Some(true),
                &matcher_main,
            )
        {
            error::quit();
        }

        // Encrypt and write changed plaintext if we need to store
        if let Some(dest) = &dest {
            // TODO: select proper recipients (use from current file?)
            let recipients = store.recipients()?;
            context
                .encrypt_file(&recipients, plaintext.clone(), &dest.0)
                .map_err(Err::Write)?;
        }

        // Copy to clipboard after editing
        #[cfg(feature = "clipboard")]
        if matcher_generate.copy() {
            clipboard::copy_plaintext(
                plaintext.clone(),
                true,
                !matcher_main.force(),
                matcher_main.quiet(),
                matcher_main.verbose(),
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
        #[cfg_attr(not(feature = "clipboard"), expect(unused_mut))]
        let mut output_any = matcher_generate.show();
        #[cfg(feature = "clipboard")]
        {
            output_any = output_any || matcher_generate.copy();
        }

        if matcher_main.verbose() || (!output_any && !matcher_main.quiet()) {
            eprintln!("Secret generated");
        }

        Ok(())
    }
}

/// Generate a random password.
///
/// This generates a secure random password/passphrase based on user configuration.
fn generate_password(matcher_generate: &GenerateMatcher) -> Plaintext {
    if matcher_generate.passphrase() {
        let config = BasicConfig {
            words: matcher_generate.length() as usize,
            ..Default::default()
        };
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

    #[error("no secret selected")]
    NoneSelected,
}
