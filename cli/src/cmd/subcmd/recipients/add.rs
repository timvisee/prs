use clap::{App, Arg};

/// The recipient add command definition.
pub struct CmdAdd;

impl CmdAdd {
    pub fn build<'a>() -> App<'a> {
        App::new("add")
            .alias("a")
            .about("Add store recipient")
            .arg(
                Arg::new("secret")
                    .long("secret")
                    .alias("private")
                    .about("Add public key we have private key for"),
            )
            .arg(
                Arg::new("no-recrypt")
                    .long("no-recrypt")
                    .alias("no-reencrypt")
                    .alias("skip-recrypt")
                    .alias("skip-reencrypt")
                    .about("Skip re-encrypting all secrets"),
            )
    }
}
