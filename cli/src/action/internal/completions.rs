use std::fs::{self, File};
use std::io;

use anyhow::Result;
use clap::ArgMatches;
use thiserror::Error;

use crate::cmd::matcher::{internal::completions::CompletionsMatcher, main::MainMatcher, Matcher};

/// A file completions action.
pub struct Completions<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Completions<'a> {
    /// Construct a new completions action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the completions action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_completions = CompletionsMatcher::with(self.cmd_matches).unwrap();

        // Obtian shells to generate completions for, build application definition
        let shells = matcher_completions.shells();
        let dir = matcher_completions.output();
        let quiet = matcher_main.quiet();
        let mut app = crate::cmd::handler::Handler::build();

        // If the directory does not exist yet, attempt to create it
        if !dir.is_dir() {
            fs::create_dir_all(&dir).map_err(Error::CreateOutputDir)?;
        }

        // Generate completions
        for shell in shells {
            if !quiet {
                eprint!(
                    "Generating completions for {}...",
                    format!("{}", shell).to_lowercase()
                );
            }
            if matcher_completions.stdout() {
                shell.generate(&mut app, matcher_completions.name(), &mut std::io::stdout());
            } else {
                // TODO: revert this to `generate_to` once clap v3.0.0-beta.3 is released, it fixes
                //       an critical issue that caused a panic. See the `clap-3.0.0-beta.3` branch.
                // shell.generate_to(&mut app, matcher_completions.name(), &dir);

                // Determine path of final file, create file, write completion script to it
                let path = dir.join(shell.file_name(&matcher_completions.name()));
                let mut file = File::create(path).map_err(Error::Write)?;
                shell.generate(&mut app, matcher_completions.name(), &mut file);
                file.sync_all().map_err(Error::Write)?;
            }
            if !quiet {
                eprintln!(" done.");
            }
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    /// An error occurred while creating the output directory.
    #[error("failed to create output directory, it doesn't exist")]
    CreateOutputDir(#[source] io::Error),

    /// An error occurred while writing completion scripts to a file.
    #[error("failed to write completion script to file")]
    Write(#[source] io::Error),
}
