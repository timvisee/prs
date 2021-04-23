use clap::{Arg, ArgMatches};

use super::{CmdArg, CmdArgOption};

/// The store argument.
pub struct ArgStore {}

impl CmdArg for ArgStore {
    fn name() -> &'static str {
        "store"
    }

    fn build<'b>() -> Arg<'b> {
        Arg::new("store")
            .long("store")
            .short('s')
            .value_name("PATH")
            .env("PASSWORD_STORE_DIR")
            .global(true)
            .about("Password store to use")
    }
}

impl<'a> CmdArgOption<'a> for ArgStore {
    type Value = String;

    fn value<'b: 'a>(matches: &'a ArgMatches) -> Self::Value {
        Self::value_raw(matches)
            .filter(|p| !p.trim().is_empty())
            .unwrap_or(prs_lib::STORE_DEFAULT_ROOT)
            .to_string()
    }
}
