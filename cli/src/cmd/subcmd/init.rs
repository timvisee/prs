use clap::{App, Arg, SubCommand};

/// The init command definition.
pub struct CmdInit;

impl CmdInit {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("init")
            .about("Initialize new password store")
            .arg(
                Arg::with_name("PATH")
                    .help("Password store path")
                    .default_value(crate::STORE_DEFAULT_ROOT)
                    .required(true),
            )
    }
}
