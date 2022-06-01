use clap::{Arg, Command};

use crate::cmd::matcher::internal::completions::Shell;

/// The generate completions command definition.
pub struct CmdCompletions;

impl CmdCompletions {
    pub fn build<'a>() -> Command<'a> {
        let shell_variants: Vec<_> = Shell::variants().iter().map(|v| v.name()).collect();

        Command::new("completions")
            .about("Shell completions")
            .alias("completion")
            .alias("complete")
            .arg(
                Arg::new("SHELL")
                    .help("Shell type to generate completions for")
                    .required(true)
                    .multiple_values(true)
                    .takes_value(true)
                    .possible_value("all")
                    .possible_values(shell_variants)
                    .ignore_case(true),
            )
            .arg(
                Arg::new("output")
                    .long("output")
                    .short('o')
                    .alias("output-dir")
                    .alias("out")
                    .alias("dir")
                    .value_name("DIR")
                    .help("Shell completion files output directory"),
            )
            .arg(
                Arg::new("stdout")
                    .long("stdout")
                    .short('s')
                    .alias("print")
                    .help("Output completion files to stdout instead")
                    .conflicts_with("output"),
            )
            .arg(
                Arg::new("name")
                    .long("name")
                    .short('n')
                    .alias("bin")
                    .alias("binary")
                    .alias("bin-name")
                    .alias("binary-name")
                    .value_name("NAME")
                    .help("Name of binary to generate completions for"),
            )
    }
}
