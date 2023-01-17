use anyhow::Result;
use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgProperty, ArgQuery, ArgTimeout, ArgViewer, CmdArgFlag, CmdArgOption};

/// The TOTP show command matcher.
pub struct ShowMatcher<'a> {
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> ShowMatcher<'a> {
    /// The secret query.
    pub fn query(&self) -> Option<String> {
        ArgQuery::value(self.matches)
    }

    /// Show timeout in seconds.
    pub fn timeout(&self) -> Option<Result<u64>> {
        ArgTimeout::value(self.matches)
    }

    /// The selected property.
    pub fn property(&self) -> Option<&String> {
        ArgProperty::value(self.matches)
    }

    /// Check whether to read from copy.
    #[cfg(feature = "clipboard")]
    pub fn copy(&self) -> bool {
        self.matches.get_flag("copy")
    }

    /// Check whether to show in a viewer.
    pub fn viewer(&self) -> bool {
        ArgViewer::is_present(self.matches) || self.timeout().is_some()
    }
}

impl<'a> Matcher<'a> for ShowMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("totp")?
            .subcommand_matches("show")
            .map(|matches| ShowMatcher { matches })
    }
}
