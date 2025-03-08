use anyhow::Result;
use bytesize::ByteSize;
use clap::ArgMatches;
use prs_lib::Store;
use thiserror::Error;

use crate::cmd::matcher::{
    tomb::{status::StatusMatcher, TombMatcher},
    MainMatcher, Matcher,
};

/// A tomb status action.
pub struct Status<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Status<'a> {
    /// Construct a new init action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the init action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let _matcher_tomb = TombMatcher::with(self.cmd_matches).unwrap();
        let matcher_status = StatusMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_main.store()).map_err(Err::Store)?;
        let tomb = store.tomb(
            !matcher_main.verbose(),
            matcher_main.verbose(),
            matcher_main.force(),
        );

        let is_tomb = tomb.is_tomb();
        if !is_tomb {
            eprintln!("Tomb: no");
            return Ok(());
        }

        // Open tomb on request
        let mut is_open = tomb.is_open().map_err(Err::Status)?;
        if matcher_status.open() && !is_open {
            if !matcher_main.quiet() {
                eprintln!("Opening password store Tomb...");
            }
            tomb.open().map_err(Err::Open)?;
            is_open = true;
        }

        let has_timer = tomb.has_timer().map_err(Err::Status)?;
        let tomb_path = tomb.find_tomb_path().unwrap();
        let tomb_key_path = tomb.find_tomb_key_path().unwrap();
        let sizes = tomb.fetch_size_stats().map_err(Err::Size)?;

        println!("Tomb: yes");
        println!("Open: {}", if is_open { "yes" } else { "no" });
        println!("Close timer: {}", if has_timer { "active" } else { "no" });
        println!("Tomb path: {}", tomb_path.display());
        println!("Tomb key path: {}", tomb_key_path.display());
        println!(
            "Store size: {}",
            sizes
                .store
                .map(|s| ByteSize(s).to_string())
                .unwrap_or_else(|| "?".into())
        );
        println!(
            "Tomb file size: {}",
            sizes
                .tomb_file
                .map(|s| ByteSize(s).to_string())
                .unwrap_or_else(|| "?".into())
        );

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[error("failed to query password store tomb status")]
    Status(#[source] anyhow::Error),

    #[error("failed to open password store tomb")]
    Open(#[source] anyhow::Error),

    #[error("failed to fetch password store size status")]
    Size(#[source] anyhow::Error),
}
