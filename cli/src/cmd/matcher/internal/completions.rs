use std::path::PathBuf;
use std::str::FromStr;

use clap::{ArgMatches, Shell};

use super::Matcher;
use crate::util;

/// The completions completions command matcher.
pub struct CompletionsMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> CompletionsMatcher<'a> {
    /// Get the shells to generate completions for.
    pub fn shells(&'a self) -> Vec<Shell> {
        // Get the raw list of shells
        let raw = self
            .matches
            .values_of("SHELL")
            .expect("no shells were given");

        // Parse the list of shell names, deduplicate
        let mut shells: Vec<_> = raw
            .into_iter()
            .map(|name| name.trim().to_lowercase())
            .map(|name| {
                if name == "all" {
                    Shell::variants()
                        .iter()
                        .map(|name| name.to_string())
                        .collect()
                } else {
                    vec![name]
                }
            })
            .flatten()
            .collect();
        shells.sort_unstable();
        shells.dedup();

        // Parse the shell names
        shells
            .into_iter()
            .map(|name| Shell::from_str(&name).expect("failed to parse shell name"))
            .collect()
    }

    /// The target directory to output the shell completion files to.
    pub fn output(&'a self) -> PathBuf {
        self.matches
            .value_of("output")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("./"))
    }

    /// Whether to print completion scripts to stdout.
    pub fn stdout(&'a self) -> bool {
        self.matches.is_present("stdout")
    }

    /// Name of binary to generate completions for.
    pub fn name(&'a self) -> String {
        self.matches
            .value_of("name")
            .map(|n| n.into())
            .unwrap_or(util::bin_name())
    }
}

impl<'a> Matcher<'a> for CompletionsMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("internal")?
            .subcommand_matches("completions")
            .map(|matches| CompletionsMatcher { matches })
    }
}
