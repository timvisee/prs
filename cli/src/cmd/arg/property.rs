use clap::{Arg, ArgMatches};

use super::{CmdArg, CmdArgOption};

/// The property argument.
pub struct ArgProperty {}

impl CmdArg for ArgProperty {
    fn name() -> &'static str {
        "property"
    }

    fn build() -> Arg {
        Arg::new("property")
            .long("property")
            .short('p')
            .alias("prop")
            .value_name("NAME")
            .num_args(1)
            .global(true)
            .help("Select a specific property")
    }
}

impl<'a> CmdArgOption<'a> for ArgProperty {
    type Value = Option<&'a String>;

    fn value(matches: &'a ArgMatches) -> Self::Value {
        Self::value_raw(matches)
    }
}
