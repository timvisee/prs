use clap::{App, Arg, SubCommand};

/// The recipient add command definition.
pub struct CmdAdd;

impl CmdAdd {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("add")
            .alias("a")
            .about("Add store recipient")
            .arg(
                Arg::with_name("secret")
                    .long("secret")
                    .alias("private")
                    .help("Add public key we have private key for"),
            )
            .arg(
                Arg::with_name("no-recrypt")
                    .long("no-recrypt")
                    .alias("no-reencrypt")
                    .alias("skip-recrypt")
                    .alias("skip-reencrypt")
                    .help("Skip re-encrypting all secrets"),
            )
    }
}
