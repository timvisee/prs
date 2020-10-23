use clap::{App, Arg, SubCommand};

use crate::cmd::arg::{ArgQuery, CmdArg};

/// The duplicate command definition.
pub struct CmdDuplicate;

impl CmdDuplicate {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("duplicate")
            .about("Duplicate secret")
            .alias("dup")
            .arg(ArgQuery::build().required(true))
            .arg(
                Arg::with_name("TARGET")
                    .help("Secret target name")
                    .required(true),
            )
    }
}
