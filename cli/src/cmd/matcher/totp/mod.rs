#[cfg(feature = "clipboard")]
pub mod copy;
pub mod live;
pub mod qr;
pub mod show;

use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgStore, CmdArgOption};

/// The TOTP command matcher.
pub struct TotpMatcher<'a> {
    root: &'a ArgMatches,
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> TotpMatcher<'a> {
    /// Get the TOTP copy sub command, if matched.
    #[cfg(feature = "clipboard")]
    pub fn cmd_copy(&'a self) -> Option<copy::CopyMatcher> {
        copy::CopyMatcher::with(self.root)
    }

    /// Get the TOTP live sub command, if matched.
    pub fn cmd_live(&'a self) -> Option<live::LiveMatcher> {
        live::LiveMatcher::with(self.root)
    }

    /// Get the TOTP QR code sub command, if matched.
    pub fn cmd_qr(&'a self) -> Option<qr::QrMatcher> {
        qr::QrMatcher::with(self.root)
    }

    /// Get the TOTP show sub command, if matched.
    pub fn cmd_show(&'a self) -> Option<show::ShowMatcher> {
        show::ShowMatcher::with(self.root)
    }

    /// The store.
    pub fn store(&self) -> String {
        ArgStore::value(self.matches)
    }
}

impl<'a> Matcher<'a> for TotpMatcher<'a> {
    fn with(root: &'a ArgMatches) -> Option<Self> {
        root.subcommand_matches("totp")
            .map(|matches| TotpMatcher { root, matches })
    }
}
