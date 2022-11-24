use clap::{Arg, Command};

/// The recipient export command definition.
pub struct CmdExport;

impl CmdExport {
    pub fn build() -> Command {
        let cmd = Command::new("export")
            .alias("exp")
            .alias("ex")
            .about("Export recipient key")
            .arg(
                Arg::new("output-file")
                    .long("output-file")
                    .short('o')
                    .alias("output")
                    .alias("file")
                    .value_name("PATH")
                    .num_args(1)
                    .help("Write recipient key to file instead of stdout"),
            );

        #[cfg(feature = "clipboard")]
        let cmd = cmd.arg(
            Arg::new("copy")
                .long("copy")
                .short('c')
                .alias("yank")
                .num_args(0)
                .help("Copy recipient key to clipboard instead of stdout"),
        );

        cmd
    }
}
