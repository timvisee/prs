use std::process::Command;

use anyhow::{anyhow, Result};
use clap::ArgMatches;
use prs_lib::Store;
use thiserror::Error;

use crate::cmd::matcher::{lock::LockMatcher, MainMatcher, Matcher};
use crate::util::error;

/// Lock password store action.
pub struct Lock<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Lock<'a> {
    /// Construct a new lock action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the lock action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_lock = LockMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_lock.store()).map_err(Err::Store)?;

        // TODO: wipe open GPG keys from RAM

        // Attempt to kill GPG agents
        kill_gpg_agent(&matcher_main);

        // Attempt to lock Tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        if let Err(err) = tomb_lock(&store, &matcher_main) {
            error::print_error(err.context("failed to close password store Tomb"));
        }

        // Attempt to invalidate cached sudo credentials
        invalidate_sudo(&matcher_main);

        // TODO: kill open persistent SSH sessions

        if !matcher_main.quiet() {
            eprintln!("Password store locked");
        }

        // TODO: show warning if not using tomb on supported platform, metadata leakage

        Ok(())
    }
}

/// Attempt to kill and clear all GPG agents that potentially unlock secrets.
fn kill_gpg_agent(matcher_main: &MainMatcher) {
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
        }
        Ok(status) if !status.success() => {
            if !matcher_main.quiet() {
                eprintln!("FAIL");
            }
            error::print_error_msg(format!(
                "failed to kill gpgconf gpg-agent (exit status: {})",
                status
            ));
        }
        Ok(_) => {
            if !matcher_main.quiet() {
                eprintln!("ok");
            }
        }
    }

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
        }
        Ok(status) if !status.success() => {
            if !matcher_main.quiet() {
                eprintln!("FAIL");
            }
            error::print_error_msg(format!(
                "failed to kill keychain GPG agent (exit status: {})",
                status
            ));
        }
        Ok(_) => {
            if !matcher_main.quiet() {
                eprintln!("ok");
            }
        }
    }

    // Kill any remaining gpg-agent processes
    // Invoke: pkill -HUP gpg-agent
    if !matcher_main.quiet() {
        eprint!("Kill other gpg-agent processes: ");
    }
    match Command::new("pkill").args(["-HUP", "gpg-agent"]).status() {
        Err(err) => {
            if !matcher_main.quiet() {
                eprintln!("FAIL");
            }
            error::print_error(anyhow!(err).context("failed to kill gpg-agent processes"));
        }
        Ok(status) if !status.success() => {
            if !matcher_main.quiet() {
                eprintln!("FAIL");
            }
            error::print_error_msg(format!(
                "failed to kill gpg-agent processes (exit status: {})",
                status
            ));
        }
        Ok(_) => {
            if !matcher_main.quiet() {
                eprintln!("ok");
            }
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

    // TODO: slam tombs

    Ok(())
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
                "failed to invalidate cached sudo credentials (exit status: {})",
                status
            ));
        }
        Ok(_) => {
            if !matcher_main.quiet() {
                eprintln!("ok");
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to close password store tomb")]
    Close(#[source] anyhow::Error),
}
