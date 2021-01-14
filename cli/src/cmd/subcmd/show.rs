use clap::{App, Arg, SubCommand};

use crate::cmd::arg::{ArgProperty, ArgQuery, ArgStore, ArgTimeout, CmdArg};

/// The show command definition.
pub struct CmdShow;

impl CmdShow {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("show")
            .alias("s")
            .alias("cat")
            .alias("display")
            .about("Display a secret")
            .arg(
                Arg::with_name("first")
                    .long("first")
                    .alias("password")
                    .alias("pass")
                    .help("Show only the first line of the secret"),
            )
            .arg(ArgQuery::build())
            .arg(ArgStore::build())
            .arg(ArgTimeout::build().help("Timeout after which to clear output"))
            .arg(ArgProperty::build().conflicts_with("first"))
    }
}
