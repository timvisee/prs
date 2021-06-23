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
        let matcher_tomb = TombMatcher::with(self.cmd_matches).unwrap();
        let matcher_status = StatusMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_tomb.store()).map_err(Err::Store)?;
        let tomb = store.tomb();

        let is_tomb = tomb.is_tomb();
        if !is_tomb {
            eprintln!("Tomb: no");
            return Ok(());
        }

        // Open tomb on requet
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

        let store_size = fs_extra::dir::get_size(&store.root).map_err(Err::StoreSize)?;
        let tomb_file_size = tomb_path.metadata().map_err(Err::TombSize)?.len();

        println!("Tomb: yes");
        println!("Open: {}", if is_open { "yes" } else { "no" });
        println!("Close timer: {}", if has_timer { "active" } else { "no" });
        println!("Tomb path: {}", tomb_path.display());
        println!("Tomb key path: {}", tomb_key_path.display());
        if is_open {
            println!("Store size: {}", ByteSize(store_size));
        } else {
            println!("Store size: ?");
        }
        println!("Tomb file size: {}", ByteSize(tomb_file_size));

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

    #[error("failed to calcualte password store size")]
    StoreSize(#[source] fs_extra::error::Error),

    #[error("failed to calcualte password store tomb size")]
    TombSize(#[source] std::io::Error),
}
