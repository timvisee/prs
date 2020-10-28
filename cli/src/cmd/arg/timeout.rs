use anyhow::Result;
use clap::{Arg, ArgMatches};
use thiserror::Error;

use super::{CmdArg, CmdArgOption};

/// The timeout argument.
pub struct ArgTimeout {}

impl CmdArg for ArgTimeout {
    fn name() -> &'static str {
        "timeout"
    }

    fn build<'b, 'c>() -> Arg<'b, 'c> {
        Arg::with_name("timeout")
            .long("timeout")
            .short("t")
            .alias("time")
            .alias("seconds")
            .alias("second")
            .value_name("SECONDS")
            .global(true)
            .help("Timeout after which to clear clipboard")
            .default_value(crate::CLIPBOARD_TIMEOUT_STR)
    }
}

impl<'a> CmdArgOption<'a> for ArgTimeout {
    type Value = Result<u64>;

    fn value<'b: 'a>(matches: &'a ArgMatches<'b>) -> Self::Value {
        Self::value_raw(matches)
            .unwrap()
            .parse()
            .map_err(|err| Err::Parse(err).into())
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to parse timeout as seconds")]
    Parse(#[source] std::num::ParseIntError),
}
