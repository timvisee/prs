use clap::{App, Arg, SubCommand};

/// The recipient export command definition.
pub struct CmdExport;

impl CmdExport {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("export")
            .alias("exp")
            .alias("ex")
            .about("Export recipient key")
            .arg(
                Arg::with_name("output-file")
                    .long("output-file")
                    .short("o")
                    .alias("output")
                    .alias("file")
                    .value_name("PATH")
                    .help("Write recipient key to file instead of stdout"),
            )
            .arg(
                Arg::with_name("copy")
                    .long("copy")
                    .short("c")
                    .alias("yank")
                    .help("Copy recipient key to clipboard instead of stdout"),
            )
    }
}
