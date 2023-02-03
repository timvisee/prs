use std::fs::{self, File};
use std::io::{self, Write};

use anyhow::Result;
use clap::{ArgMatches, Command};
use clap_complete::shells;
use thiserror::Error;

use crate::cmd::matcher::{
    internal::completions::{CompletionsMatcher, Shell},
    main::MainMatcher,
    Matcher,
};

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
                    format!("{shell}").to_lowercase()
                );
            }
            if matcher_completions.stdout() {
                generate(
                    shell,
                    &mut app,
                    matcher_completions.name(),
                    &mut std::io::stdout(),
                );
            } else {
                // Determine path of final file, create file, write completion script to it
                let path = dir.join(shell.file_name(&matcher_completions.name()));
                let mut file = File::create(path).map_err(Error::Write)?;
                generate(shell, &mut app, matcher_completions.name(), &mut file);
                file.sync_all().map_err(Error::Write)?;
            }
            if !quiet {
                eprintln!(" done.");
            }
        }

        Ok(())
    }
}

/// Generate completion script.
fn generate<S>(shell: Shell, app: &mut Command, bin_name: S, buf: &mut dyn Write)
where
    S: Into<String>,
{
    match shell {
        Shell::Bash => {
            let mut inner_buf = Vec::new();
            clap_complete::generate(shells::Bash, app, bin_name, &mut inner_buf);

            // Patch bash completion to complete secret names
            let inner_buf = String::from_utf8(inner_buf)
                .expect("clap_complete::generate should always return valid utf-8")
                .replace("<QUERY>", "$(prs list --list --quiet)")
                .replace("[QUERY]", "$(prs list --list --quiet)");

            buf.write_fmt(format_args!("{inner_buf}"))
                .expect("failed to write to generated file"); // Same panic that clap_complete would trigger
        }
        // TODO: patch other completion scripts to complete secret names like with bash
        Shell::Elvish => clap_complete::generate(shells::Elvish, app, bin_name, buf),
        Shell::Fish => clap_complete::generate(shells::Fish, app, bin_name, buf),
        Shell::PowerShell => clap_complete::generate(shells::PowerShell, app, bin_name, buf),
        Shell::Zsh => clap_complete::generate(shells::Zsh, app, bin_name, buf),
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
