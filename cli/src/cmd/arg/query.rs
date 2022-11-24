use clap::{Arg, ArgMatches};

use super::{CmdArg, CmdArgOption};

/// The query argument.
pub struct ArgQuery {}

impl CmdArg for ArgQuery {
    fn name() -> &'static str {
        "QUERY"
    }

    fn build() -> Arg {
        Arg::new("QUERY").help("Secret query")
    }
}

impl<'a> CmdArgOption<'a> for ArgQuery {
    type Value = Option<String>;

    fn value(matches: &'a ArgMatches) -> Self::Value {
        let parts: Vec<String> = Self::values_raw(matches)?.map(|s| s.to_string()).collect();
        Some(parts.join(" "))
    }
}
