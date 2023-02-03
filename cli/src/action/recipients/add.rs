use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{crypto::prelude::*, Recipients, Store};
use thiserror::Error;

use crate::cmd::matcher::{
    recipients::{add::AddMatcher, RecipientsMatcher},
    MainMatcher, Matcher,
};
#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;
use crate::util::{self, error, select, style, sync};

/// A recipients add action.
pub struct Add<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Add<'a> {
    /// Construct a new add action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the add action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let _matcher_recipients = RecipientsMatcher::with(self.cmd_matches).unwrap();
        let matcher_add = AddMatcher::with(self.cmd_matches).unwrap();

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

        // Prepare sync
        sync::ensure_ready(&sync, matcher_add.allow_dirty());
        if !matcher_add.no_sync() {
            sync.prepare()?;
        }

        let mut context = crate::crypto::context(&matcher_main)?;
        let mut recipients = store.recipients().map_err(Err::Load)?;

        // Find unused keys, select one and add to recipients
        let mut tmp = Recipients::from(
            if !matcher_add.secret() {
                context.keys_public()
            } else {
                context.keys_private()
            }
            .map_err(Err::Load)?,
        );
        tmp.remove_all(recipients.keys());
        let key = select::select_key(tmp.keys(), None).ok_or(Err::NoneSelected)?;
        recipients.add(key.clone());
        recipients.save(&store)?;

        if prs_lib::store::can_decrypt(&store) {
            // Recrypt secrets
            // TODO: do not quit on error, finish sync, ask to revert instead?
            if !matcher_add.no_recrypt() {
                crate::action::housekeeping::recrypt::recrypt_all(&store, &matcher_main)
                    .map_err(Err::Recrypt)?;
            }
        } else if !matcher_main.quiet() {
            cannot_decrypt_show_recrypt_hints();
        }

        // Finalize sync
        sync.finalize(format!("Add recipient {}", key.fingerprint(true)))?;

        // Finalize tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::finalize_tomb(&mut tomb, &matcher_main, true).map_err(Err::Tomb)?;

        if !matcher_main.quiet() {
            eprintln!("Added recipient: {key}");
        }

        Ok(())
    }
}

/// Cannot decrypt on this machine, show recrypt hints.
pub(crate) fn cannot_decrypt_show_recrypt_hints() {
    // TODO: only show this if adding secret key
    error::print_warning("cannot read secrets on this machine");
    error::print_warning("re-encrypt secrets on another machine with this store to fix");

    let bin = util::bin_name();
    println!();
    println!("Run this on another machine to re-encrypt secrets:");
    println!(
        "    {}",
        style::highlight(&format!("{bin} housekeeping recrypt --all"))
    );
    println!();
    println!("When done, pull in the re-encrypted secrets here with:");
    println!("    {}", style::highlight(&format!("{bin} sync")));
    println!();
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[cfg(all(feature = "tomb", target_os = "linux"))]
    #[error("failed to prepare password store tomb for usage")]
    Tomb(#[source] anyhow::Error),

    #[error("no key selected")]
    NoneSelected,

    #[error("failed to load usable keys from keychain")]
    Load(#[source] anyhow::Error),

    #[error("failed to re-encrypt secrets in store")]
    Recrypt(#[source] anyhow::Error),
}
