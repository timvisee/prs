use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{
    crypto::{self, prelude::*},
    Recipients, Store,
};
use thiserror::Error;

use crate::{
    cmd::matcher::{
        tomb::{init::InitMatcher, TombMatcher},
        MainMatcher, Matcher,
    },
    util::{
        self,
        error::{self, ErrorHintsBuilder},
        select, style,
    },
};

/// A tomb init action.
pub struct Init<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Init<'a> {
    /// Construct a new init action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the init action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_tomb = TombMatcher::with(self.cmd_matches).unwrap();
        let matcher_init = InitMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_tomb.store()).map_err(Err::Store)?;
        let tomb = store.tomb(!matcher_main.verbose(), matcher_main.verbose());

        // Must not be a tomb already
        if tomb.is_tomb() && !matcher_main.force() {
            error::quit_error_msg(
                "password store already is a tomb",
                ErrorHintsBuilder::default().force(true).build().unwrap(),
            );
        }

        // Select GPG key to encrypt Tomb key
        let mut context = crypto::context(crypto::PROTO)?;
        let tmp = Recipients::from(context.keys_private().map_err(Err::Load)?);
        let key =
            select::select_key(tmp.keys(), Some("Select key for Tomb")).ok_or(Err::NoGpgKey)?;

        // TODO: ask user to add selected key to recipients if not yet part of it?

        // Initialize tomb
        tomb.init(key).map_err(Err::Init)?;

        // Run housekeeping
        crate::action::housekeeping::run::housekeeping(
            &store,
            matcher_init.allow_dirty(),
            matcher_init.no_sync(),
        )
        .map_err(Err::Housekeeping)?;

        if !matcher_main.quiet() {
            // if let Some(timer) = timer {
            //     eprintln!(
            //         "Password store Tomb opened, will close in {}",
            //         util::time::format_duration(timer)
            //     );
            // } else {
            eprintln!("Password store Tomb initialized and opened");
            // }
            eprintln!("");
            eprintln!("To close the Tomb, use:");
            eprintln!(
                "    {}",
                style::highlight(&format!("{} tomb close", util::bin_name()))
            );
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to initialize tomb")]
    Init(#[source] anyhow::Error),

    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to run housekeeping tasks")]
    Housekeeping(#[source] anyhow::Error),

    #[error("failed to load usable keys from keychain")]
    Load(#[source] anyhow::Error),

    #[error("no GPG key selected to create Tomb")]
    NoGpgKey,
}
