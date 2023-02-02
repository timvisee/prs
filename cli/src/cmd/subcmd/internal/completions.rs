use clap::{Arg, Command};

use crate::cmd::matcher::internal::completions::Shell;

/// The generate completions command definition.
pub struct CmdCompletions;

impl CmdCompletions {
    pub fn build() -> Command {
        let mut shell_variants = vec!["all"];
        shell_variants.extend(Shell::variants().iter().map(|v| v.name()));

        Command::new("completions")
            .about("Shell completions")
            .alias("completion")
            .alias("complete")
            .arg(
                Arg::new("SHELL")
                    .help("Shell type to generate completions for")
                    .required(true)
                    .num_args(1..)
                    // TODO: replace this by a runtime list
                    // Issue: https://github.com/clap-rs/clap/issues/4504#issuecomment-1326379595
                    // .value_parser(shell_variants)
                    .value_parser(["all", "bash", "zsh", "fish", "elvish", "powershell"])
                    .ignore_case(true),
            )
            .arg(
                Arg::new("output")
                    .long("output")
                    .short('o')
                    .alias("output-dir")
                    .alias("out")
                    .alias("dir")
                    .num_args(1)
                    .value_name("DIR")
                    .help("Shell completion files output directory"),
            )
            .arg(
                Arg::new("stdout")
                    .long("stdout")
                    .alias("print")
                    .num_args(0)
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
                    .num_args(1)
                    .help("Name of binary to generate completions for"),
            )
    }
}
