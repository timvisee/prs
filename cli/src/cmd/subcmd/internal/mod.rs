#[cfg(feature = "clipboard")]
pub mod clip_revert;
pub mod completions;

use clap::Command;

/// The internal command definition.
pub struct CmdInternal;

impl CmdInternal {
    pub fn build() -> Command {
        #[allow(unused_mut)]
        let mut cmd = Command::new("internal")
            .about("Commands used by prs internally")
            .hide(true)
            .subcommand_required(true)
            .subcommand_value_name("ACTION")
            .subcommand(completions::CmdCompletions::build());

        #[cfg(feature = "clipboard")]
        {
            cmd = cmd.subcommand(clip_revert::CmdClipRevert::build());
        }

        cmd
    }
}
