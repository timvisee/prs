use anyhow::Result;
use clap::{Arg, ArgMatches};
use thiserror::Error;

use super::{CmdArg, CmdArgOption};

/// The timeout argument.
pub struct ArgTimeout {}

impl ArgTimeout {
    #[cfg(feature = "clipboard")]
    pub fn value_or_default(matches: &ArgMatches) -> Result<u64> {
        Self::value(matches).unwrap_or(Ok(crate::CLIPBOARD_TIMEOUT))
    }
}

impl CmdArg for ArgTimeout {
    fn name() -> &'static str {
        "timeout"
    }

    fn build<'b>() -> Arg<'b> {
        Arg::new("timeout")
            .long("timeout")
            .short('t')
            .alias("time")
            .alias("seconds")
            .alias("second")
            .value_name("SECONDS")
            .global(true)
            .help("Timeout after which to clear clipboard")
    }
}

impl<'a> CmdArgOption<'a> for ArgTimeout {
    type Value = Option<Result<u64>>;

    fn value(matches: &'a ArgMatches) -> Self::Value {
        Self::value_raw(matches).map(|t| t.parse().map_err(|err| Err::Parse(err).into()))
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to parse timeout as seconds")]
    Parse(#[source] std::num::ParseIntError),
}
