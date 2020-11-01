use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::store::Store;

use crate::cmd::matcher::{
    recipients::{add::AddMatcher, RecipientsMatcher},
    MainMatcher, Matcher,
};
use crate::util::{self, error, skim, style, sync};

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
        let sync = store.sync();

        sync::ensure_ready(&sync);
        sync.prepare()?;

        let mut recipients = store.recipients().map_err(Err::Load)?;

        // Find unused keys, select one and add to recipients
        let mut tmp = prs_lib::all(matcher_add.secret()).map_err(Err::Load)?;
        tmp.remove_many(recipients.keys());
        let key = skim::skim_select_key(tmp.keys()).ok_or(Err::NoneSelected)?;
        recipients.add(key.clone());

        recipients.save(&store)?;

        if prs_lib::store::can_decrypt(&store) {
            // Recrypt secrets
            // TODO: do not quit on error, finish sync, ask to revert instead?
            if !matcher_add.no_recrypt() {
                crate::action::housekeeping::recrypt::recrypt_all(
                    &store,
                    matcher_main.quiet(),
                    matcher_main.verbose(),
                )
                .map_err(Err::Recrypt)?;
            }
        } else {
            if !matcher_main.quiet() {
                cannot_decrypt_show_recrypt_hints();
            }
        }

        sync.finalize(format!("Add recipient {}", key.fingerprint(true)))?;

        if !matcher_main.quiet() {
            eprintln!("Added recipient: {}", key);
        }

        Ok(())
    }
}

/// Cannot decrypt on this machine, show recrypt hints.
// TODO: move this somewhere central
pub(crate) fn cannot_decrypt_show_recrypt_hints() {
    // TODO: only show this if adding secret key
    error::print_warning("cannot read secrets on this machine");
    error::print_warning("re-encrypt secrets on another machine with this store to fix");

    let bin = util::bin_name();
    println!();
    println!("Run this on another machine to re-encrypt secrets:");
    println!(
        "    {}",
        style::highlight(&format!("{} housekeeping recrypt --all", bin))
    );
    println!();
    println!("When done, pull in the re-encrypted secrets here with:");
    println!("    {}", style::highlight(&format!("{} sync", bin)));
    println!();
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
