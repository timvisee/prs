use clap::{App, Arg, SubCommand};

/// The generate command definition.
pub struct CmdGenerate;

impl CmdGenerate {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("generate")
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
                Arg::with_name("copy")
                    .long("copy")
                    .short("c")
                    .alias("cp")
                    .help("Copy secret to clipboard"),
            )
            .arg(
                Arg::with_name("show")
                    .long("show")
                    .short("s")
                    .alias("cat")
                    .alias("display")
                    .help("Display secret after generation"),
            )
    }
}
