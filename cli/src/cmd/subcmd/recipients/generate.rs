use clap::{App, Arg};

/// The recipient generate command definition.
pub struct CmdGenerate;

impl CmdGenerate {
    pub fn build<'a>() -> App<'a> {
        App::new("generate")
            .alias("gen")
            .alias("g")
            .about("Generate new key pair, add it to the store")
            .arg(
                Arg::new("no-add")
                    .long("no-add")
                    .alias("skip-add")
                    .about("Skip adding key pair to store"),
            )
            .arg(
                Arg::new("no-recrypt")
                    .long("no-recrypt")
                    .alias("no-reencrypt")
                    .alias("skip-recrypt")
                    .alias("skip-reencrypt")
                    .about("Skip re-encrypting all secrets")
                    .conflicts_with("no-add"),
            )
    }
}
