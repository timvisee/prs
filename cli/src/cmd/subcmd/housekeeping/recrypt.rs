use clap::{App, Arg, SubCommand};

use crate::cmd::arg::{ArgQuery, CmdArg};

/// The housekeeping recrypt command definition.
pub struct CmdRecrypt;

impl CmdRecrypt {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("recrypt")
            .alias("reencrypt")
            .about("Re-encrypt secrets")
            .arg(
                Arg::with_name("all")
                    .long("all")
                    .short("a")
                    .help("Re-encrypt all secrets")
                    .conflicts_with("QUERY"),
            )
            .arg(ArgQuery::build().required_unless("all"))
    }
}
