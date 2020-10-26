use clap::{App, Arg, SubCommand};

/// The new command definition.
pub struct CmdNew;

impl CmdNew {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("new")
            .alias("n")
            .alias("create")
            .about("Create new secret")
            .arg(
                Arg::with_name("DEST")
                    .help("Secret destination path")
                    .required(true),
            )
            .arg(
                Arg::with_name("empty")
                    .long("empty")
                    .short("e")
                    .help("Create empty secret, do not edit"),
            )
            .arg(
                Arg::with_name("stdin")
                    .long("stdin")
                    .short("S")
                    .alias("from-stdin")
                    .help("Read secret from stdin, do not open editor")
                    .conflicts_with("empty"),
            )
    }
}
