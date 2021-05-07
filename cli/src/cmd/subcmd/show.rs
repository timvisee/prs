use clap::{App, Arg};

use crate::cmd::arg::{ArgProperty, ArgQuery, ArgStore, ArgTimeout, CmdArg};

/// The show command definition.
pub struct CmdShow;

impl CmdShow {
    pub fn build<'a>() -> App<'a> {
        let cmd = App::new("show")
            .alias("s")
            .alias("cat")
            .alias("display")
            .alias("print")
            .about("Display a secret")
            .arg(
                Arg::new("first")
                    .long("first")
                    .alias("password")
                    .alias("pass")
                    .about("Show only the first line of the secret"),
            )
            .arg(ArgQuery::build())
            .arg(ArgStore::build())
            .arg(ArgTimeout::build().about("Timeout after which to clear output"))
            .arg(ArgProperty::build().conflicts_with("first"));

        #[cfg(feature = "clipboard")]
        let cmd = cmd
            .arg(
                Arg::new("copy")
                    .long("copy")
                    .short('c')
                    .alias("cp")
                    .about("Copy secret to clipboard"),
            )
            .arg(ArgTimeout::build().requires("copy"));

        cmd
    }
}
