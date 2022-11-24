use clap::{Arg, Command};

#[cfg(feature = "clipboard")]
use crate::cmd::arg::ArgTimeout;
use crate::cmd::arg::{ArgAllowDirty, ArgNoSync, ArgStore, CmdArg};

/// The generate command definition.
pub struct CmdGenerate;

impl CmdGenerate {
    pub fn build() -> Command {
        let cmd = Command::new("generate")
            .alias("gen")
            .alias("g")
            .alias("random")
            .about("Generate a secure secret")
            .arg(
                Arg::new("NAME")
                    .help("Secret name and path")
                    .required_unless_present_any(["show", "copy"]),
            )
            .arg(
                Arg::new("passphrase")
                    .long("passphrase")
                    .short('P')
                    .num_args(0)
                    .help("Generate passhprase instead of random string"),
            )
            .arg(
                Arg::new("length")
                    .value_name("NUM")
                    .long("length")
                    .short('l')
                    .alias("len")
                    .num_args(1)
                    .help("Generated password length in characters")
                    .long_help(
                        "Generated password length in characters. Passphrase length in words.",
                    ),
            )
            .arg(
                Arg::new("merge")
                    .long("merge")
                    .short('m')
                    .num_args(0)
                    .help("Merge into existing secret, don't create new secret"),
            )
            .arg(
                Arg::new("edit")
                    .long("edit")
                    .short('e')
                    .num_args(0)
                    .help("Edit secret after generation"),
            )
            .arg(
                Arg::new("stdin")
                    .long("stdin")
                    .short('S')
                    .alias("from-stdin")
                    .num_args(0)
                    .help("Append to generated secret from stdin")
                    .conflicts_with("edit"),
            )
            .arg(
                Arg::new("show")
                    .long("show")
                    .alias("cat")
                    .alias("display")
                    .num_args(0)
                    .help("Display secret after generation"),
            )
            .arg(ArgStore::build())
            .arg(ArgAllowDirty::build())
            .arg(ArgNoSync::build());

        #[cfg(feature = "clipboard")]
        let cmd = cmd
            .arg(
                Arg::new("copy")
                    .long("copy")
                    .short('c')
                    .alias("cp")
                    .num_args(0)
                    .help("Copy secret to clipboard"),
            )
            .arg(ArgTimeout::build().requires("copy"));

        cmd
    }
}
