use std::fmt;
use std::io::Write;
use std::path::PathBuf;

use clap::{ArgMatches, Command};
use clap_complete::shells;

use super::Matcher;
use crate::util;

/// The completions completions command matcher.
pub struct CompletionsMatcher<'a> {
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> CompletionsMatcher<'a> {
    /// Get the shells to generate completions for.
    pub fn shells(&'a self) -> Vec<Shell> {
        // Get the raw list of shells
        let raw = self
            .matches
            .get_many("SHELL")
            .expect("no shells were given");

        // Parse the list of shell names, deduplicate
        let mut shells: Vec<_> = raw
            .into_iter()
            .map(|name: &String| name.trim().to_lowercase())
            .flat_map(|name| {
                if name == "all" {
                    Shell::variants().iter().map(|s| s.name().into()).collect()
                } else {
                    vec![name]
                }
            })
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
            .get_one("output")
            .map(|v: &String| PathBuf::from(v))
            .unwrap_or_else(|| PathBuf::from("./"))
    }

    /// Whether to print completion scripts to stdout.
    pub fn stdout(&'a self) -> bool {
        self.matches.get_flag("stdout")
    }

    /// Name of binary to generate completions for.
    pub fn name(&'a self) -> String {
        self.matches
            .get_one("name")
            .map(String::clone)
            .unwrap_or_else(util::bin_name)
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

/// Available shells.
#[derive(Copy, Clone)]
pub enum Shell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

impl Shell {
    /// List all supported shell variants.
    pub fn variants() -> &'static [Shell] {
        &[
            Shell::Bash,
            Shell::Elvish,
            Shell::Fish,
            Shell::PowerShell,
            Shell::Zsh,
        ]
    }

    /// Select shell variant from name.
    pub fn from_str(shell: &str) -> Option<Shell> {
        match shell.trim().to_ascii_lowercase().as_str() {
            "bash" => Some(Shell::Bash),
            "elvish" => Some(Shell::Elvish),
            "fish" => Some(Shell::Fish),
            "powershell" | "ps" => Some(Shell::PowerShell),
            "zsh" => Some(Shell::Zsh),
            _ => None,
        }
    }

    /// Get shell name.
    pub fn name(self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Elvish => "elvish",
            Shell::Fish => "fish",
            Shell::PowerShell => "powershell",
            Shell::Zsh => "zsh",
        }
    }

    /// Suggested file name for completions file of current shell.
    pub fn file_name(self, bin_name: &str) -> String {
        match self {
            Shell::Bash => format!("{}.bash", bin_name),
            Shell::Elvish => format!("{}.elv", bin_name),
            Shell::Fish => format!("{}.fish", bin_name),
            Shell::PowerShell => format!("_{}.ps1", bin_name),
            Shell::Zsh => format!("_{}", bin_name),
        }
    }

    /// Generate completion script.
    pub fn generate<S>(self, app: &mut Command, bin_name: S, buf: &mut dyn Write)
    where
        S: Into<String>,
    {
        match self {
            Shell::Bash => {
                let mut inner_buf = Vec::new();
                clap_complete::generate(shells::Bash, app, bin_name, &mut inner_buf);

                let inner_buf = String::from_utf8(inner_buf)
                    .expect("clap_complete::generate should always return valid utf-8");

                let inner_buf = inner_buf.replace("<QUERY>", "$(prs list --list)");

                buf.write_fmt(format_args!("{}", inner_buf))
                    .expect("Failed to write to generated file"); // Same panic that clap_complete would trigger
            }
            Shell::Elvish => clap_complete::generate(shells::Elvish, app, bin_name, buf),
            Shell::Fish => clap_complete::generate(shells::Fish, app, bin_name, buf),
            Shell::PowerShell => clap_complete::generate(shells::PowerShell, app, bin_name, buf),
            Shell::Zsh => clap_complete::generate(shells::Zsh, app, bin_name, buf),
        }
    }

    // /// Generate completion script.
    // pub fn generate_to<S, T>(self, app: &mut Command<'_>, bin_name: S, out_dir: T)
    // where
    //     S: Into<String>,
    //     T: Into<std::ffi::OsString>,
    // {
    //     match self {
    //         Shell::Bash => {
    //             clap_complete::generate_to::<shells::Bash, _, _>(app, bin_name, out_dir)
    //         }
    //         Shell::Elvish => {
    //             clap_complete::generate_to::<shells::Elvish, _, _>(app, bin_name, out_dir)
    //         }
    //         Shell::Fish => {
    //             clap_complete::generate_to::<shells::Fish, _, _>(app, bin_name, out_dir)
    //         }
    //         Shell::PowerShell => {
    //             clap_complete::generate_to::<shells::PowerShell, _, _>(app, bin_name, out_dir)
    //         }
    //         Shell::Zsh => {
    //             clap_complete::generate_to::<shells::Zsh, _, _>(app, bin_name, out_dir)
    //         }
    //     }
    // }
}

impl fmt::Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
