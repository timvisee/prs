use clap::{Arg, ArgMatches};

use super::{CmdArg, CmdArgOption};

/// The store argument.
pub struct ArgStore {}

impl CmdArg for ArgStore {
    fn name() -> &'static str {
        "store"
    }

    fn build() -> Arg {
        Arg::new("store")
            .long("store")
            .short('s')
            .value_name("PATH")
            .env("PASSWORD_STORE_DIR")
            .num_args(1)
            .global(true)
            .help("Password store to use")
    }
}

impl<'a> CmdArgOption<'a> for ArgStore {
    type Value = String;

    fn value(matches: &'a ArgMatches) -> Self::Value {
        Self::value_raw(matches)
            .filter(|p| !p.trim().is_empty())
            .unwrap_or(&prs_lib::STORE_DEFAULT_ROOT.into())
            .to_string()
    }
}
