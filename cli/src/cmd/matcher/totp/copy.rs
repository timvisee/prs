use anyhow::Result;
use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgProperty, ArgQuery, ArgTimeout, CmdArgOption};

/// The TOTP copy command matcher.
pub struct CopyMatcher<'a> {
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> CopyMatcher<'a> {
    /// Don't recopy if the token changes within the timeout.
    pub fn no_recopy(&self) -> bool {
        self.matches.get_flag("no-recopy")
    }

    /// The secret query.
    pub fn query(&self) -> Option<String> {
        ArgQuery::value(self.matches)
    }

    /// Clipboard timeout in seconds.
    pub fn timeout(&self) -> Option<Result<u64>> {
        ArgTimeout::value(self.matches)
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
