#[cfg(feature = "clipboard")]
pub mod copy;
pub mod live;
pub mod qr;
pub mod show;

use clap::ArgMatches;

use super::Matcher;

/// The TOTP command matcher.
pub struct TotpMatcher<'a> {
    root: &'a ArgMatches,
    _matches: &'a ArgMatches,
}

impl<'a> TotpMatcher<'a> {
    /// Get the TOTP copy sub command, if matched.
    #[cfg(feature = "clipboard")]
    pub fn cmd_copy(&'a self) -> Option<copy::CopyMatcher<'a>> {
        copy::CopyMatcher::with(self.root)
    }

    /// Get the TOTP live sub command, if matched.
    pub fn cmd_live(&'a self) -> Option<live::LiveMatcher<'a>> {
        live::LiveMatcher::with(self.root)
    }

    /// Get the TOTP QR code sub command, if matched.
    pub fn cmd_qr(&'a self) -> Option<qr::QrMatcher<'a>> {
        qr::QrMatcher::with(self.root)
    }

    /// Get the TOTP show sub command, if matched.
    pub fn cmd_show(&'a self) -> Option<show::ShowMatcher<'a>> {
        show::ShowMatcher::with(self.root)
    }
}

impl<'a> Matcher<'a> for TotpMatcher<'a> {
    fn with(root: &'a ArgMatches) -> Option<Self> {
        root.subcommand_matches("totp").map(|matches| TotpMatcher {
            root,
            _matches: matches,
        })
    }
}
