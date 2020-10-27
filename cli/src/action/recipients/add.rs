use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::store::Store;

use crate::cmd::matcher::{
    recipients::{add::AddMatcher, RecipientsMatcher},
    MainMatcher, Matcher,
};
use crate::util;

/// A recipients add action.
pub struct Add<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Add<'a> {
    /// Construct a new add action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the add action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_recipients = RecipientsMatcher::with(self.cmd_matches).unwrap();
        let matcher_add = AddMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_recipients.store()).map_err(Err::Store)?;
        let mut recipients = store.recipients().map_err(Err::Load)?;

        // Find unused keys, select one and add to recipients
        let mut tmp = prs_lib::all().map_err(Err::Load)?;
        tmp.remove_many(recipients.keys());
        let key = util::skim_select_key(tmp.keys()).ok_or(Err::NoneSelected)?;
        recipients.add(key.clone());

        recipients.save(&store)?;

        // Recrypt secrets
        if !matcher_add.no_recrypt() {
            crate::action::housekeeping::recrypt::recrypt_all(
                &store,
                matcher_main.quiet(),
                matcher_main.verbose(),
            )
            .map_err(Err::Recrypt)?;
        }

        // TODO: sync

        if !matcher_main.quiet() {
            eprintln!("Added recipient: {}", key);
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("no key selected")]
    NoneSelected,

    #[error("failed to load usable keys from keychain")]
    Load(#[source] anyhow::Error),

    #[error("failed to re-encrypt secrets in store")]
    Recrypt(#[source] anyhow::Error),
}
