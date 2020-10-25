use std::io::Write;

use clap::ArgMatches;

use anyhow::Result;
use prs_lib::{store::Store, types::Plaintext};
use thiserror::Error;

use crate::cmd::matcher::{show::ShowMatcher, Matcher};

/// A file show action.
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
        let matcher_show = ShowMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(crate::STORE_DEFAULT_ROOT);

        let secrets = store.secrets(matcher_show.query());
        let secret = crate::select_secret(&secrets).ok_or(Err::NoneSelected)?;

        // TODO: attach decrypt error here
        let mut plaintext =
            prs_lib::crypto::decrypt_file(&secret.path).map_err(|_| Err::Decrypt)?;

        // Trim plaintext to first line
        if matcher_show.first_line() {
            plaintext = plaintext.first_line().map_err(Err::FirstLine)?;
        }

        print(plaintext)
    }
}

/// Print the given plaintext to stdout.
fn print(plaintext: Plaintext) -> Result<()> {
    eprintln!("=v=v=v=v=v=v=v=v=v=");
    std::io::stdout()
        .write_all(&plaintext.0)
        .map_err(Err::Print)?;
    let _ = std::io::stdout().flush();
    eprintln!("\n=^=^=^=^=^=^=^=^=^=");
    Ok(())
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to grab first line of secret")]
    FirstLine(#[source] std::str::Utf8Error),

    #[error("no secret selected")]
    NoneSelected,

    #[error("failed to decrypt secret")]
    Decrypt,

    #[error("failed to print secret to stdout")]
    Print(#[source] std::io::Error),
}
