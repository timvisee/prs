use std::fs;
use std::path::Path;

use anyhow::Result;
use clap::ArgMatches;
use prs_lib::{crypto, Store};
use thiserror::Error;

use crate::cmd::matcher::{init::InitMatcher, MainMatcher, Matcher};
use crate::util::{self, style};
use crate::vendor::shellexpand;

/// Init store action.
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
        let matcher_init = InitMatcher::with(self.cmd_matches).unwrap();

        let path = shellexpand::full(&matcher_init.store())
            .map_err(Err::ExpandPath)?
            .to_string();

        // Ensure store dir is free, then initialize
        util::fs::ensure_dir_free(&Path::new(&path))?;
        fs::create_dir_all(&path).map_err(Err::Init)?;

        // Open new store
        let store = Store::open(&path).map_err(Err::Store)?;

        // Run housekeeping
        crate::action::housekeeping::run::housekeeping(&store, true, false)
            .map_err(Err::Housekeeping)?;

        // Hint user to add our recipient key
        if !matcher_main.quiet() {
            let bin = util::bin_name();
            let system_has_secret = crypto::util::has_private_key(crypto::PROTO).unwrap_or(true);

            if system_has_secret {
                eprintln!("Now add your own key as recipient or generate a new one:");
            } else {
                eprintln!("Now generate and add a new recipient key for yourself:");
            }
            if system_has_secret {
                eprintln!(
                    "    {}",
                    style::highlight(&format!("{} recipients add --secret", bin))
                );
            }
            eprintln!(
                "    {}",
                style::highlight(&format!("{} recipients generate", bin))
            );
            eprintln!();
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to expand store path")]
    ExpandPath(#[source] shellexpand::LookupError<std::env::VarError>),

    #[error("failed to initialize store")]
    Init(#[source] std::io::Error),

    #[error("failed to access initialized password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to run housekeeping tasks")]
    Housekeeping(#[source] anyhow::Error),
}
