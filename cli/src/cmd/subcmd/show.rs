use clap::{Arg, Command};

use crate::cmd::arg::{ArgProperty, ArgQuery, ArgTimeout, CmdArg};

/// The show command definition.
pub struct CmdShow;

impl CmdShow {
    pub fn build() -> Command {
        let cmd = Command::new("show")
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
                    .num_args(0)
                    .help("Show only the first line of the secret"),
            )
            .arg(ArgQuery::build())
            .arg(
                ArgTimeout::build()
                    .help("Timeout after which to clear output")
                    .conflicts_with("no-interact"),
            )
            .arg(ArgProperty::build().conflicts_with("first"));

        #[cfg(feature = "clipboard")]
        let cmd = cmd.arg(
            Arg::new("copy")
                .long("copy")
                .short('c')
                .alias("cp")
                .num_args(0)
                .help("Copy secret to clipboard"),
        );

        cmd
    }
}
