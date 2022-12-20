use clap::{Arg, Command};

use crate::cmd::arg::{ArgProperty, ArgQuery, ArgTimeout, CmdArg};

/// The TOTP show command definition.
pub struct CmdShow;

impl CmdShow {
    pub fn build() -> Command {
        let cmd = Command::new("show")
            .alias("s")
            .alias("cat")
            .alias("display")
            .alias("print")
            .about("Show TOTP token")
            .arg(ArgQuery::build())
            .arg(ArgTimeout::build().help("Timeout after which to clear output"))
            .arg(ArgProperty::build());

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
