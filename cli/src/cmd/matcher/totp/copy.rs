use anyhow::Result;
use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgProperty, ArgQuery, ArgTimeout, CmdArgOption};

/// The TOTP copy command matcher.
pub struct CopyMatcher<'a> {
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> CopyMatcher<'a> {
    /// The secret query.
    pub fn query(&self) -> Option<String> {
        ArgQuery::value(self.matches)
    }

    /// Clipboard timeout in seconds.
    pub fn timeout(&self) -> Result<u64> {
        ArgTimeout::value_or_default(self.matches)
    }

    /// The selected property.
    pub fn property(&self) -> Option<&String> {
        ArgProperty::value(self.matches)
    }
}

impl<'a> Matcher<'a> for CopyMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("totp")?
            .subcommand_matches("copy")
            .map(|matches| CopyMatcher { matches })
    }
}
