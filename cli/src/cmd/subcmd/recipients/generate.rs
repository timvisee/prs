use clap::{App, Arg, SubCommand};

/// The recipient generate command definition.
pub struct CmdGenerate;

impl CmdGenerate {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("generate")
            .alias("gen")
            .alias("g")
            .about("Generate new key pair, add it to the store")
            .arg(
                Arg::with_name("skip-add")
                    .long("skip-add")
                    .alias("no-add")
                    .help("Skip adding key pair to store"),
            )
    }
}
