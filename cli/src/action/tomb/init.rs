use anyhow::{anyhow, Result};
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
        self, cli,
        error::{self, ErrorHintsBuilder},
        select, style, sync,
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
        let sync = store.sync();
        let tomb = store.tomb(!matcher_main.verbose(), matcher_main.verbose());
        let timer = matcher_init.timer();

        // Must not be a tomb already
        if tomb.is_tomb() && !matcher_main.force() {
            error::quit_error_msg(
                "password store already is a tomb",
                ErrorHintsBuilder::default().force(true).build().unwrap(),
            );
        }

        // Ask user to confirm
        eprintln!("This will create a new Tomb and will move your current password store into it.");
        if !cli::prompt_yes(
            "Are you sure you want to continue?",
            Some(true),
            &matcher_main,
        ) {
            if matcher_main.verbose() {
                eprintln!("Tomb initialisation cancelled");
            }
            error::quit();
        }

        // Select GPG key to encrypt Tomb key
        let mut context = crypto::context(crypto::PROTO)?;
        let tmp = Recipients::from(context.keys_private().map_err(Err::Load)?);
        let key =
            select::select_key(tmp.keys(), Some("Select key for Tomb")).ok_or(Err::NoGpgKey)?;

        // Prepare sync
        sync::ensure_ready(&sync, matcher_init.allow_dirty());
        if !matcher_init.no_sync() {
            sync.prepare()?;
        }

        // TODO: ask user to add selected key to recipients if not yet part of it?
        // TODO: ask user for preferred size, must be 10+ MB

        // Determine Tomb size (prefer twice size of current store in MBs)
        let mbs = util::fs::dir_size(&store.root)
            .map(|bytes| ((bytes * 2) / (1024 * 1024)).max(10) as u32)
            .unwrap_or_else(|err| {
                error::print_error(
                    anyhow!(err).context("failed to calcualte password store size, assuming 10MB"),
                );
                10
            });

        if !matcher_main.quiet() {
            eprintln!("Initializing Tomb, this may take a while...");
            eprintln!("");
        }

        // Initialize tomb
        tomb.init(key, mbs).map_err(Err::Init)?;

        // Finalize sync
        if !matcher_init.no_sync() {
            sync.finalize("Initialize Tomb")?;
        }

        // Run housekeeping
        crate::action::housekeeping::run::housekeeping(
            &store,
            matcher_init.allow_dirty(),
            matcher_init.no_sync(),
        )
        .map_err(Err::Housekeeping)?;

        // Start timer
        if let Some(timer) = timer {
            if let Err(err) = tomb.stop_timer() {
                error::print_error(err.context(
                    "failed to stop existing timer to automatically close password store tomb, ignoring",
                ));
            }
            tomb.start_timer(timer, true).map_err(Err::Timer)?;
        }

        if !matcher_main.quiet() {
            eprintln!("");
            if let Some(timer) = timer {
                eprintln!(
                    "Password store Tomb initialized and opened, will close in {}",
                    util::time::format_duration(timer)
                );
            } else {
                eprintln!("Password store Tomb initialized and opened");
            }
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

    #[error("failed to start timer to automatically close password store tomb")]
    Timer(#[source] anyhow::Error),
}
