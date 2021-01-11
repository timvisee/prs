#[cfg(feature = "clipboard")]
use anyhow::Result;
use clap::ArgMatches;

use super::Matcher;
#[cfg(feature = "clipboard")]
use crate::cmd::arg::ArgTimeout;
use crate::cmd::arg::{ArgStore, CmdArgOption};

/// Default password length in characters.
const PASSWORD_LENGTH: u16 = 24;

/// Default passphrase length in words.
const PASSPHRASE_LENGTH: u16 = 5;

/// The generate command matcher.
pub struct GenerateMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> GenerateMatcher<'a> {
    /// Secret destination.
    pub fn destination(&self) -> Option<&str> {
        self.matches.value_of("DEST")
    }

    /// Check whether to generate a passphrase.
    pub fn passphrase(&self) -> bool {
        self.matches.is_present("passphrase")
    }

    /// What length to use.
    pub fn length(&self) -> u16 {
        self.matches
            .value_of("length")
            .map(|l| l.parse().expect("invalid length"))
            .unwrap_or_else(|| {
                if self.passphrase() {
                    PASSPHRASE_LENGTH
                } else {
                    PASSWORD_LENGTH
                }
            })
    }

    /// Check whether to edit the secret.
    pub fn edit(&self) -> bool {
        self.matches.is_present("edit")
    }

    /// Check whether to read from stdin.
    pub fn stdin(&self) -> bool {
        self.matches.is_present("stdin")
    }

    /// Check whether to read from copy.
    #[cfg(feature = "clipboard")]
    pub fn copy(&self) -> bool {
        self.matches.is_present("copy")
    }

    /// Clipboard timeout in seconds.
    #[cfg(feature = "clipboard")]
    pub fn timeout(&self) -> Result<u64> {
        ArgTimeout::value(self.matches)
    }

    /// Check whether to read from show.
    pub fn show(&self) -> bool {
        self.matches.is_present("show")
    }

    /// The store.
    pub fn store(&self) -> String {
        ArgStore::value(self.matches)
    }
}

impl<'a> Matcher<'a> for GenerateMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("generate")
            .map(|matches| GenerateMatcher { matches })
    }
}
