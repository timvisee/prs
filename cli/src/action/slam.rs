use std::process::Command;

use anyhow::{anyhow, Result};
use clap::ArgMatches;
#[cfg(unix)]
use prs_lib::util::git;
use prs_lib::Store;
use thiserror::Error;

use crate::cmd::matcher::{slam::SlamMatcher, MainMatcher, Matcher};
use crate::util::error;

#[cfg(all(feature = "tomb", target_os = "linux"))]
use prs_lib::tomb::{self, TombSettings};

/// Slam password store action.
pub struct Slam<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Slam<'a> {
    /// Construct a new slam action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the slam action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let _matcher_slam = SlamMatcher::with(self.cmd_matches).unwrap();

        // Attempt to open store for some locking operations
        let store = match Store::open(matcher_main.store()) {
            Ok(store) => Some(store),
            Err(err) => {
                error::print_error(Err::Store(err).into());
                None
            }
        };

        // Attempt to flush GPG agents
        flush_gpg_agents(&matcher_main);

        // Attempt to lock Tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        if let Some(store) = &store {
            if let Err(err) = tomb_lock(store, &matcher_main) {
                error::print_error(Err::Close(err).into());
            }
        }

        // Attempt to invalidate cached sudo credentials
        match has_bin("sudo") {
            Ok(true) => invalidate_sudo(&matcher_main),
            Ok(false) => {}
            Err(err) => error::print_error(err.context("failed to invalidate sudo credentials")),
        }
        match has_bin("doas") {
            Ok(true) => invalidate_doas(&matcher_main),
            Ok(false) => {}
            Err(err) => error::print_error(err.context("failed to invalidate doas credentials")),
        }

        // Drop open prs persistent SSH sessions
        #[cfg(unix)]
        if let Some(store) = &store {
            drop_persistent_ssh(store, &matcher_main);
        }

        if !matcher_main.quiet() {
            eprintln!("Password store locked");
        }

        Ok(())
    }
}

/// Attempt to flush and clear all GPG agents that potentially unlock secrets.
fn flush_gpg_agents(matcher_main: &MainMatcher) {
    let mut flushed = false;

    // Kill GPG agent through gpgconf
    match has_bin("gpgconf") {
        Ok(true) => flushed = gpgconf_kill(matcher_main) || flushed,
        Ok(false) => {}
        Err(err) => error::print_error(err.context("failed to kill GPG agent through gpgconf")),
    }

    // Clear GPG agent through keychain
    match has_bin("keychain") {
        Ok(true) => flushed = keychain_clear(matcher_main) || flushed,
        Ok(false) => {}
        Err(err) => error::print_error(err.context("failed to clear GPG agent through keychain")),
    }

    // Reload GPG agents through pkill
    #[cfg(unix)]
    match has_bin("pkill") {
        Ok(true) => flushed = pkill_reload_gpgagent(matcher_main) || flushed,
        Ok(false) => {}
        Err(err) => error::print_error(err.context("failed to reload GPG agents through pkill")),
    }

    // Show warning if not flushed
    if !flushed {
        error::print_warning("no GPG agent is flushed, cleared or killed");
    }
}

/// Kill GPG agent using gpgconf.
fn gpgconf_kill(matcher_main: &MainMatcher) -> bool {
    // Signal gpg-agent kill through gpgconf
    // Invoke: gpgconf --kill gpg-agent
    if !matcher_main.quiet() {
        eprint!("Signal gpgconf gpg-agent kill: ");
    }
    match Command::new("gpgconf")
        .args(["--kill", "gpg-agent"])
        .status()
    {
        Err(err) => {
            if !matcher_main.quiet() {
                eprintln!("FAIL");
            }
            error::print_error(anyhow!(err).context("failed to kill gpgconf gpg-agent"));
            false
        }
        Ok(status) if !status.success() => {
            if !matcher_main.quiet() {
                eprintln!("FAIL");
            }
            error::print_error_msg(format!(
                "failed to kill gpgconf gpg-agent (exit status: {status})"
            ));
            false
        }
        Ok(_) => {
            if !matcher_main.quiet() {
                eprintln!("ok");
            }
            true
        }
    }
}

/// Clear GPG agent through keychain.
fn keychain_clear(matcher_main: &MainMatcher) -> bool {
    // Signal to clear keychain GPG agent
    // Invoke: keychain --quiet --clear --agents gpg
    if !matcher_main.quiet() {
        eprint!("Clear keychain GPG agent: ");
    }
    match Command::new("keychain")
        .args(["--quiet", "--clear", "--agents", "gpg"])
        .status()
    {
        Err(err) => {
            if !matcher_main.quiet() {
                eprintln!("FAIL");
            }
            error::print_error(anyhow!(err).context("failed to kill keychain GPG agent"));
            false
        }
        Ok(status) if !status.success() => {
            if !matcher_main.quiet() {
                eprintln!("FAIL");
            }
            error::print_error_msg(format!(
                "failed to kill keychain GPG agent (exit status: {status})",
            ));
            false
        }
        Ok(_) => {
            if !matcher_main.quiet() {
                eprintln!("ok");
            }
            true
        }
    }
}

/// Reload configuration of gpg-agent processes.
#[cfg(unix)]
fn pkill_reload_gpgagent(matcher_main: &MainMatcher) -> bool {
    // Kill any remaining gpg-agent processes
    // Invoke: pkill -HUP gpg-agent
    if !matcher_main.quiet() {
        eprint!("Reload gpg-agent processes: ");
    }
    match Command::new("pkill").args(["-HUP", "gpg-agent"]).status() {
        Ok(status) if status.code() == Some(0) => {
            if !matcher_main.quiet() {
                eprintln!("ok");
            }
            true
        }
        Ok(status) if status.code() == Some(1) => {
            if !matcher_main.quiet() {
                eprintln!("ok");
            }
            false
        }
        Ok(_) => {
            if !matcher_main.quiet() {
                eprintln!("FAIL");
            }
            false
        }
        Err(err) => {
            if !matcher_main.quiet() {
                eprintln!("FAIL");
            }
            error::print_error(anyhow!(err).context("failed to reload gpg-agent processes"));
            false
        }
    }
}

/// Attempt to lock Tomb.
#[cfg(all(feature = "tomb", target_os = "linux"))]
fn tomb_lock(store: &Store, matcher_main: &MainMatcher) -> Result<()> {
    let tomb = store.tomb(
        !matcher_main.verbose(),
        matcher_main.verbose(),
        matcher_main.force(),
    );

    // Must be a tomb, must be open, assume it is
    if !tomb.is_tomb() || !tomb.is_open().unwrap_or(true) {
        return Ok(());
    }

    // Close the tomb
    if !matcher_main.quiet() {
        eprint!("Close Tomb: ");
    }
    match tomb.close() {
        Err(err) => {
            if !matcher_main.quiet() {
                eprintln!("FAIL");
            }
            return Err(Err::Close(err).into());
        }
        Ok(_) => {
            if !matcher_main.quiet() {
                eprintln!("ok");
            }
        }
    }

    // Close any running close timers
    if let Err(err) = tomb.stop_timer() {
        error::print_error(err.context("failed to stop auto closing systemd timer, ignoring"));
    }

    // If the Tomb is still open, slam all open Tombs
    if tomb.is_open().unwrap_or(false) {
        tomb_slam(matcher_main)?;
    }

    Ok(())
}

/// Attempt to slam Tombs.
#[cfg(all(feature = "tomb", target_os = "linux"))]
fn tomb_slam(matcher_main: &MainMatcher) -> Result<()> {
    let tomb_settings = TombSettings {
        quiet: matcher_main.quiet(),
        verbose: matcher_main.verbose(),
        force: matcher_main.force(),
    };

    // Slam open tombs
    if !matcher_main.quiet() {
        eprint!("Slam Tombs: ");
    }
    match tomb::slam(tomb_settings) {
        Err(err) => {
            if !matcher_main.quiet() {
                eprintln!("FAIL");
            }
            Err(Err::Slam(err).into())
        }
        Ok(_) => {
            if !matcher_main.quiet() {
                eprintln!("ok");
            }
            Ok(())
        }
    }
}

/// Attempt to invalidate cached sudo credentials that are still active.
fn invalidate_sudo(matcher_main: &MainMatcher) {
    if !matcher_main.quiet() {
        eprint!("Invalidate cached sudo credentials: ");
    }
    match Command::new("sudo").args(["-K"]).status() {
        Err(err) => {
            if !matcher_main.quiet() {
                eprintln!("FAIL");
            }
            error::print_error(
                anyhow!(err).context("failed to invalidate cached sudo credentials"),
            );
        }
        Ok(status) if !status.success() => {
            if !matcher_main.quiet() {
                eprintln!("FAIL");
            }
            error::print_error_msg(format!(
                "failed to invalidate cached sudo credentials (exit status: {status})",
            ));
        }
        Ok(_) => {
            if !matcher_main.quiet() {
                eprintln!("ok");
            }
        }
    }
}

/// Attempt to invalidate cached doas credentials that are still active.
fn invalidate_doas(matcher_main: &MainMatcher) {
    if !matcher_main.quiet() {
        eprint!("Invalidate cached doas credentials: ");
    }
    match Command::new("doas").args(["-L"]).status() {
        Err(err) => {
            if !matcher_main.quiet() {
                eprintln!("FAIL");
            }
            error::print_error(
                anyhow!(err).context("failed to invalidate cached doas credentials"),
            );
        }
        Ok(status) if !status.success() => {
            if !matcher_main.quiet() {
                eprintln!("FAIL");
            }
            error::print_error_msg(format!(
                "failed to invalidate cached doas credentials (exit status: {status})",
            ));
        }
        Ok(_) => {
            if !matcher_main.quiet() {
                eprintln!("ok");
            }
        }
    }
}

/// Drop any open prs persistent SSH sessions.
#[cfg(unix)]
fn drop_persistent_ssh(store: &Store, matcher_main: &MainMatcher) {
    if !matcher_main.quiet() {
        eprint!("Drop persistent SSH sessions: ");
    }

    // Kill any still open
    git::kill_ssh_by_session(store);

    if !matcher_main.quiet() {
        eprintln!("ok");
    }
}

/// Check if the given binary is found and is invocable.
fn has_bin(bin: &str) -> Result<bool> {
    match which::which(bin) {
        Ok(_) => Ok(true),
        Err(which::Error::CannotFindBinaryPath) => Ok(false),
        Err(err) => Err(Err::ProbeBinary(err, bin.into()).into()),
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to find binary: {1}")]
    ProbeBinary(#[source] which::Error, String),

    #[cfg(all(feature = "tomb", target_os = "linux"))]
    #[error("failed to close password store tomb")]
    Close(#[source] anyhow::Error),

    #[cfg(all(feature = "tomb", target_os = "linux"))]
    #[error("failed to slam open tombs")]
    Slam(#[source] anyhow::Error),
}
