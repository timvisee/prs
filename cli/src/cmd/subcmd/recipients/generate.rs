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
                Arg::with_name("no-add")
                    .long("no-add")
                    .alias("skip-add")
                    .help("Skip adding key pair to store"),
            )
            .arg(
                Arg::with_name("no-recrypt")
                    .long("no-recrypt")
                    .alias("no-reencrypt")
                    .alias("skip-recrypt")
                    .alias("skip-reencrypt")
                    .help("Skip re-encrypting all secrets")
                    .conflicts_with("no-add"),
            )
    }
}
