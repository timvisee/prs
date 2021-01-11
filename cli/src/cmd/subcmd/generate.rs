use clap::{App, Arg, SubCommand};

#[cfg(feature = "clipboard")]
use crate::cmd::arg::ArgTimeout;
use crate::cmd::arg::{ArgStore, CmdArg};

/// The generate command definition.
pub struct CmdGenerate;

impl CmdGenerate {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        let cmd = SubCommand::with_name("generate")
            .alias("gen")
            .alias("g")
            .alias("random")
            .about("Generate a secure secret")
            .arg(
                Arg::with_name("DEST")
                    .help("Secret destination path")
                    .required(true),
            )
            .arg(
                Arg::with_name("passphrase")
                    .long("passphrase")
                    .short("P")
                    .help("Generate passhprase instead of random string"),
            )
            .arg(
                Arg::with_name("length")
                    .value_name("NUM")
                    .long("length")
                    .short("l")
                    .alias("len")
                    .help("Generated password length in characters")
                    .long_help(
                        "Generated password length in characters. Passphrase length in words.",
                    ),
            )
            .arg(
                Arg::with_name("edit")
                    .long("edit")
                    .short("e")
                    .help("Edit secret after generation"),
            )
            .arg(
                Arg::with_name("stdin")
                    .long("stdin")
                    .short("S")
                    .alias("from-stdin")
                    .help("Append to generated secret from stdin")
                    .conflicts_with("edit"),
            )
            .arg(
                Arg::with_name("show")
                    .long("show")
                    .alias("cat")
                    .alias("display")
                    .help("Display secret after generation"),
            )
            .arg(ArgStore::build());

        #[cfg(feature = "clipboard")]
        let cmd = cmd
            .arg(
                Arg::with_name("copy")
                    .long("copy")
                    .short("c")
                    .alias("cp")
                    .help("Copy secret to clipboard"),
            )
            .arg(ArgTimeout::build());

        cmd
    }
}
