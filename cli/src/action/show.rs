use std::thread;
use std::time::Duration;

use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{
    crypto::{self, prelude::*},
    Store,
};
use thiserror::Error;

use crate::cmd::matcher::{show::ShowMatcher, MainMatcher, Matcher};
use crate::util::{secret, select};

/// Show secret action.
pub struct Show<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Show<'a> {
    /// Construct a new show action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
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

        secret::print_name(matcher_show.query(), &secret, &store, matcher_main.quiet());

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

        secret::print(plaintext).map_err(Err::Print)?;

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
