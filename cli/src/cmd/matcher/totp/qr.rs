use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgProperty, ArgQuery, CmdArgOption};

/// The TOTP QR code command matcher.
pub struct QrMatcher<'a> {
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> QrMatcher<'a> {
    /// The secret query.
    pub fn query(&self) -> Option<String> {
        ArgQuery::value(self.matches)
    }

    /// The selected property.
    pub fn property(&self) -> Option<&String> {
        ArgProperty::value(self.matches)
    }
}

impl<'a> Matcher<'a> for QrMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("totp")?
            .subcommand_matches("qr")
            .map(|matches| QrMatcher { matches })
    }
}
