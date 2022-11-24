use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, CmdArgFlag};
use crate::util::error::{quit_error, ErrorHints};

/// The tomb init command matcher.
pub struct InitMatcher<'a> {
    matches: &'a ArgMatches,
}

impl<'a: 'b, 'b> InitMatcher<'a> {
    /// The time to automatically close.
    pub fn timer(&self) -> Option<u32> {
        let time: &String = self.matches.get_one("timer")?;
        match crate::util::time::parse_duration(time) {
            Ok(0) => None,
            Ok(time) => Some(time as u32),
            Err(err) => quit_error(err.into(), ErrorHints::default()),
        }
    }

    /// Whether to allow a dirty repository for syncing.
    pub fn allow_dirty(&self) -> bool {
        ArgAllowDirty::is_present(self.matches)
    }

    /// Whether to not sync.
    pub fn no_sync(&self) -> bool {
        ArgNoSync::is_present(self.matches)
    }
}

impl<'a> Matcher<'a> for InitMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("tomb")?
            .subcommand_matches("init")
            .map(|matches| InitMatcher { matches })
    }
}
