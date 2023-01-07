use clap::{Arg, Command};

/// The git command definition.
pub struct CmdGit;

impl CmdGit {
    pub fn build() -> Command {
        Command::new("git")
            .about("Invoke git command in password store")
            .arg(
                Arg::new("COMMAND")
                    .help("Git command to invoke")
                    .num_args(..),
            )
            .trailing_var_arg(true)
    }
}
