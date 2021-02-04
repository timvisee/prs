use std::io::Write;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{
    crypto::{self, prelude::*},
    Plaintext, Store,
};
use thiserror::Error;

use crate::cmd::matcher::{show::ShowMatcher, MainMatcher, Matcher};
use crate::util::select;

/// Show secret action.
pub struct Show<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Show<'a> {
    /// Construct a new show action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the show action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_show = ShowMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_show.store()).map_err(Err::Store)?;
        let secret =
            select::store_select_secret(&store, matcher_show.query()).ok_or(Err::NoneSelected)?;

        let mut plaintext = crypto::context(crypto::PROTO)?
            .decrypt_file(&secret.path)
            .map_err(Err::Read)?;

        // Trim plaintext to first line or property
        if matcher_show.first_line() {
            plaintext = plaintext.first_line()?;
        } else if let Some(property) = matcher_show.property() {
            plaintext = plaintext.property(property).map_err(Err::Property)?;
        }

        let lines = plaintext.unsecure_to_str().unwrap().lines().count();

        print(plaintext)?;

        // Clear after timeout
        if let Some(timeout) = matcher_show.timeout() {
            let timeout = timeout?;
            let mut lines = lines as u16 + 1;

            if matcher_main.verbose() {
                lines += 2;
                eprintln!();
                eprint!("Clearing output in {} seconds...", timeout);
            }

            thread::sleep(Duration::from_secs(timeout));
            eprint!("{}", ansi_escapes::EraseLines(lines));
        }

        Ok(())
    }
}

/// Print the given plaintext to stdout.
// TODO: move to shared module
pub(crate) fn print(plaintext: Plaintext) -> Result<()> {
    let mut stdout = std::io::stdout();

    stdout
        .write_all(plaintext.unsecure_ref())
        .map_err(Err::Print)?;

    // Always finish with newline
    if let Some(&last) = plaintext.unsecure_ref().last() {
        if last != b'\n' {
            stdout.write_all(&[b'\n']).map_err(Err::Print)?;
        }
    }

    let _ = stdout.flush();
    Ok(())
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("no secret selected")]
    NoneSelected,

    #[error("failed to read secret")]
    Read(#[source] anyhow::Error),

    #[error("failed to print secret to stdout")]
    Print(#[source] std::io::Error),

    #[error("failed to select property from secret")]
    Property(#[source] anyhow::Error),
}
