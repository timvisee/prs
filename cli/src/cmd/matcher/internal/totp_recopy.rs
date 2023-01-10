use anyhow::Result;
use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::ArgTimeout;

/// The internal TOTP recopy command matcher.
pub struct TotpRecopyMatcher<'a> {
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> TotpRecopyMatcher<'a> {
    /// Clipboard timeout in seconds.
    pub fn timeout(&self) -> Result<u64> {
        ArgTimeout::value_or_default(self.matches)
    }
}

impl<'a> Matcher<'a> for TotpRecopyMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("internal")?
            .subcommand_matches("totp-recopy")
            .map(|matches| TotpRecopyMatcher { matches })
    }
}
