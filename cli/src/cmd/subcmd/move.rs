use clap::{Arg, Command};

use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, ArgQuery, CmdArg};

/// The move command definition.
pub struct CmdMove;

impl CmdMove {
    pub fn build() -> Command {
        #[cfg_attr(not(feature = "alias"), expect(clippy::let_and_return))]
        let cmd = Command::new("move")
            .alias("mov")
            .alias("mv")
            .alias("rename")
            .alias("ren")
            .about("Move a secret")
            .arg(ArgQuery::build().required(true))
            .arg(
                Arg::new("DEST")
                    .help("Secret destination path")
                    .required(true),
            )
            .arg(ArgAllowDirty::build())
            .arg(ArgNoSync::build());

        #[cfg(feature = "alias")]
        let cmd = cmd.arg(
            Arg::new("alias")
                .long("alias")
                .short('A')
                .num_args(0)
                .help("Create alias/symlink at old path pointing to new location"),
        );

        cmd
    }
}
