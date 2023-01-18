use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{
    crypto::{prelude::*, Context},
    store::SecretIterConfig,
    Plaintext, Secret, Store,
};
use regex::Regex;
use thiserror::Error;

use crate::cmd::matcher::{grep::GrepMatcher, MainMatcher, Matcher};
#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;
use crate::util::{
    error::{self, ErrorHints, ErrorHintsBuilder},
    progress::{self, ProgressBarExt},
};

/// Maximum number of failures without forcing.
const MAX_FAIL: usize = 4;

/// Grep secrets action.
pub struct Grep<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Grep<'a> {
    /// Construct a new grep action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the grep action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_grep = GrepMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_main.store()).map_err(Err::Store)?;
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        let mut tomb = store.tomb(
            !matcher_main.verbose(),
            matcher_main.verbose(),
            matcher_main.force(),
        );

        // Prepare tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::prepare_tomb(&mut tomb, &matcher_main).map_err(Err::Tomb)?;

        // Grep aliases based on filters, sort the list
        let config = SecretIterConfig {
            find_files: true,
            find_symlink_files: matcher_grep.with_aliases(),
        };
        let mut secrets: Vec<Secret> = store
            .secret_iter_config(config)
            .filter_name(matcher_grep.query())
            .collect();
        secrets.sort_unstable_by(|a, b| a.name.cmp(&b.name));

        // Return none selected error if we have an empty list
        if secrets.is_empty() {
            return Err(Err::NoSecret.into());
        }

        grep(
            &secrets,
            &matcher_grep.pattern(),
            &matcher_main,
            &matcher_grep,
        )?;

        // Finalize tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::finalize_tomb(&mut tomb, &matcher_main, false).map_err(Err::Tomb)?;

        Ok(())
    }
}

/// Grep the given secrets.
fn grep(
    secrets: &[Secret],
    pattern: &str,
    matcher_main: &MainMatcher,
    matcher_grep: &GrepMatcher,
) -> Result<()> {
    let mut context = crate::crypto::context(matcher_main)?;
    let (mut found, mut failed) = (0, 0);

    // Parse regex if enabled
    let regex = if matcher_grep.regex() {
        Some(Regex::new(pattern).map_err(Err::Regex)?)
    } else {
        None
    };

    // Progress bar
    let pb = progress::progress_bar(secrets.len() as u64, matcher_main.quiet());

    for secret in secrets.iter() {
        pb.set_message_trunc(&secret.name);

        // Parse normally or with regex
        let result = match &regex {
            Some(re) => grep_single_regex(&mut context, secret, re),
            None => grep_single(&mut context, secret, pattern),
        };

        // Grep single secret
        match result {
            Ok(true) => {
                pb.println_always(&secret.name);
                found += 1;
            }
            Ok(false) => {}
            Err(err) => {
                error::print_error(err.context(format!("failed to grep: {}", secret.name)));
                failed += 1;
            }
        }

        pb.inc(1);

        // Stop after many failures
        if failed > MAX_FAIL && !matcher_main.force() {
            error::quit_error_msg(
                format!("stopped after {} failures", failed),
                ErrorHintsBuilder::from_matcher(matcher_main)
                    .force(true)
                    .build()
                    .unwrap(),
            );
        }
    }

    pb.finish_and_clear();

    if !matcher_main.quiet() {
        if found > 0 {
            eprintln!();
            eprintln!("Found {} of {} matches", found, secrets.len());
        } else {
            eprintln!("No matches in {} secrets", secrets.len());
        }
    }

    if failed > 0 {
        error::quit_error_msg(
            format!("Failed to grep {} of {} secrets", failed, secrets.len()),
            ErrorHints::default(),
        );
    }

    Ok(())
}

/// Grep a single secret.
fn grep_single(context: &mut Context, secret: &Secret, pattern: &str) -> Result<bool> {
    let plaintext: Plaintext = context
        .decrypt_file(&secret.path)
        .map_err(Err::Read)?
        .unsecure_to_str()
        .map_err(Err::Utf8)?
        .to_uppercase()
        .into();

    Ok(plaintext
        .unsecure_to_str()
        .unwrap()
        .contains(&pattern.to_uppercase()))
}

/// Grep a single secret using a regular expression.
fn grep_single_regex(context: &mut Context, secret: &Secret, pattern: &Regex) -> Result<bool> {
    let plaintext = context.decrypt_file(&secret.path).map_err(Err::Read)?;

    Ok(pattern.is_match(plaintext.unsecure_to_str().map_err(Err::Utf8)?))
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("no secret to grep")]
    NoSecret,

    #[cfg(all(feature = "tomb", target_os = "linux"))]
    #[error("failed to prepare password store tomb for usage")]
    Tomb(#[source] anyhow::Error),

    #[error("failed to parse pattern as regular expression")]
    Regex(#[source] regex::Error),

    #[error("failed to read secret")]
    Read(#[source] anyhow::Error),

    #[error("failed to parse secret contents as UTF-8")]
    Utf8(#[source] std::str::Utf8Error),
}
