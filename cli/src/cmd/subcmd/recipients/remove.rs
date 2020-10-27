use clap::{App, Arg, SubCommand};

/// The recipient remove command definition.
pub struct CmdRemove;

impl CmdRemove {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("remove")
            .alias("rm")
            .alias("delete")
            .alias("del")
            .about("Remove store recipient")
            .arg(
                Arg::with_name("recrypt")
                    .long("recrypt")
                    .alias("reencrypt")
                    .help("Re-encrypting all secrets"),
            )
    }
}
