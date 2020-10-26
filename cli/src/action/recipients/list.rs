use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use gpgme::Key;
use prs_lib::store::Store;

use crate::cmd::matcher::{recipients::RecipientsMatcher, Matcher};

/// A recipients list action.
pub struct List<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> List<'a> {
    /// Construct a new list action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the list action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_recipients = RecipientsMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_recipients.store()).map_err(Err::Store)?;
        let recipients = store.recipients().map_err(Err::List)?;

        recipients.keys().iter().for_each(print_recipient);

        Ok(())
    }
}

/// Print information for a recipient.
fn print_recipient(key: &Key) {
    // Generate user name/comment/email string
    let user_data = key
        .user_ids()
        .map(|user| {
            let mut parts = vec![];
            if let Ok(name) = user.name() {
                if !name.trim().is_empty() {
                    parts.push(name.into());
                }
            }
            if let Ok(comment) = user.comment() {
                if !comment.trim().is_empty() {
                    parts.push(format!("({})", comment));
                }
            }
            if let Ok(email) = user.email() {
                if !email.trim().is_empty() {
                    parts.push(format!("<{}>", email));
                }
            }
            parts.join(" ")
        })
        .collect::<Vec<_>>()
        .join("; ");

    println!("{} - {}", key.fingerprint().unwrap_or("?"), user_data)
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to list store recipients")]
    List(#[source] anyhow::Error),
}
