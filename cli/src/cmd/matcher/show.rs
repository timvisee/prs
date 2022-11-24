use anyhow::Result;
use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgProperty, ArgQuery, ArgStore, ArgTimeout, CmdArgOption};

/// The show command matcher.
pub struct ShowMatcher<'a> {
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> ShowMatcher<'a> {
    /// Check whether to just show the first line of the secret.
    pub fn first_line(&self) -> bool {
        self.matches.get_flag("first")
    }

    /// The secret query.
    pub fn query(&self) -> Option<String> {
        ArgQuery::value(self.matches)
    }

    /// The store.
    pub fn store(&self) -> String {
        ArgStore::value(self.matches)
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
}

impl<'a> Matcher<'a> for ShowMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("show")
            .map(|matches| ShowMatcher { matches })
    }
}
