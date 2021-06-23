use std::ffi::OsStr;
use std::process::{Command, ExitStatus, Output};

use anyhow::Result;
use thiserror::Error;

/// sudo binary.
pub const SUDO_BIN: &str = "sudo";

/// systemd-run binary.
pub const SYSTEMD_RUN_BIN: &str = "systemd-run";

/// Spawn systemd timer to run the given command.
///
/// This spawns as root.
pub fn systemd_cmd_timer(time: u32, description: &str, unit: &str, cmd: &[&str]) -> Result<()> {
    // TODO: do not set -q flag if in verbose mode?
    let time = format!("{}", time);
    let mut systemd_cmd = vec![
        "--quiet",
        "--system",
        "--on-active",
        &time,
        "--description",
        description,
        "--unit",
        unit,
        "--",
    ];
    systemd_cmd.extend(cmd);
    systemd_run(&systemd_cmd)
}

/// Invoke a tomb command with the given arguments.
///
/// The command will take over the user console for in/output.
fn systemd_run<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    cmd_assert_status(cmd_systemd_run(args).status().map_err(Err::System)?)
}

/// Build a systemd-run command to run.
fn cmd_systemd_run<I, S>(args: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(SUDO_BIN);
    cmd.arg("--");
    cmd.arg(SYSTEMD_RUN_BIN);
    cmd.args(args);
    cmd
}

/// Assert the exit status of a command.
///
/// Returns error is status is not succesful.
fn cmd_assert_status(status: ExitStatus) -> Result<()> {
    if !status.success() {
        return Err(Err::Status(status).into());
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to complete tomb operation")]
    Other(#[source] std::io::Error),

    #[error("failed to complete tomb operation")]
    TombCli(#[source] anyhow::Error),

    #[error("failed to invoke system command")]
    System(#[source] std::io::Error),

    #[error("tomb operation exited with non-zero status code: {0}")]
    Status(std::process::ExitStatus),
}
