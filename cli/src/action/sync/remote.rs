use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use prs_lib::{store::Store, sync::Readyness};

use crate::{
    cmd::matcher::{
        sync::{remote::RemoteMatcher, SyncMatcher},
        MainMatcher, Matcher,
    },
    util::error,
};

/// A sync remote action.
pub struct Remote<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Remote<'a> {
    /// Construct a new remote action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the remote action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_sync = SyncMatcher::with(self.cmd_matches).unwrap();
        let matcher_remote = RemoteMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_sync.store()).map_err(Err::Store)?;
        let sync = store.sync();

        // Do not set remote if sync is not initialized
        match sync.readyness()? {
            Readyness::NoSync => {
                // TODO: show as error?
                println!("Sync not configured, to initialize use: prs sync init");
                crate::util::error::quit();
            }
            _ => {}
        }

        // Get or set remote
        let repo = &store.root;
        let remotes = prs_lib::git::git_remote(repo)?;
        match matcher_remote.git_url() {
            Some(url) => {
                match remotes.len() {
                    0 => {
                        prs_lib::git::git_remote_add_url(repo, "origin", url)?;
                    }
                    1 => {
                        prs_lib::git::git_remote_set_url(repo, &remotes[0], url)?;
                    }
                    _ => {
                        eprintln!("Multiple remotes available, cannot set automatically, inspect using: prs git remote");
                        error::quit();
                    }
                }
                if !matcher_main.quiet() {
                    eprintln!("Sync remote set");
                }
            }
            None => match remotes.len() {
                0 => eprintln!("No remote configured"),
                1 => {
                    let url = prs_lib::git::git_remote_get_url(repo, &remotes[0])?;
                    eprintln!("{}", url);
                }
                _ => eprintln!("Multiple remotes configured, inspect using: prs git remote"),
            },
        }

        // TODO: sync if remote is changed?

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    // TODO: add relevant errors here
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),
}
