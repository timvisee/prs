use std::ffi::OsStr;
use std::process::{Command, ExitStatus, Stdio};

use anyhow::Result;
use thiserror::Error;

/// sudo binary.
pub const SUDO_BIN: &str = "sudo";

/// systemd-run binary.
pub const SYSTEMD_RUN_BIN: &str = "systemd-run";

/// systemctl binary.
pub const SYSTEMCTL_BIN: &str = "systemctl";

/// Spawn systemd timer to run the given command.
///
/// This may ask for root privileges through sudo.
pub fn systemd_cmd_timer(time: u32, description: &str, unit: &str, cmd: &[&str]) -> Result<()> {
    // Remove unit first if it failed before
    let _ = systemd_remove_timer(unit);
    let _ = systemctl_reset_failed_timer(unit);

    // TODO: do not set -q flag if in verbose mode?
    let time = format!("{}", time);
    let mut systemd_cmd = vec![
        "--quiet",
        "--system",
        "--on-active",
        &time,
        "--timer-property=AccuracySec=1s",
        "--description",
        description,
        "--unit",
        unit,
        "--",
    ];
    systemd_cmd.extend(cmd);
    systemd_run(&systemd_cmd)
}

/// Reset a given failed unit.
///
/// This errors if the given unit is unknown, or if it didn't fail.
///
/// Because this involves timers, if this operation fails it will also internally try to do the
/// same for the given unit with a `.timer` suffix.
fn systemctl_reset_failed_timer(unit: &str) -> Result<()> {
    // Invoke command, collect result
    let result = cmd_systemctl(&["--quiet", "--system", "reset-failed", unit])
        .stderr(Stdio::null())
        .status()
        .map_err(Err::Systemctl);

    // Do the same with .timer unit suffix, ensure one succeeds
    if unit.ends_with(".service") {
        let unit = format!("{}.timer", unit.trim_end_matches(".service"));
        systemctl_reset_failed_timer(&unit).or_else(|_| cmd_assert_status(result?))
    } else {
        cmd_assert_status(result?)
    }
}

/// Check whether the given unit (transient timer) is running.
///
/// This may ask for root privileges through sudo.
///
/// Because this involves timers, if this operation fails it will also internally try to do the
/// same for the given unit with a `.timer` suffix.
pub fn systemd_has_timer(unit: &str) -> Result<bool> {
    // TODO: check whether we can optimize this, the status command may be expensive
    let cmd = cmd_systemctl(&["--system", "--no-pager", "--quiet", "status", unit])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(Err::Systemctl)?;

    // Check special status codes
    match cmd.code() {
        Some(0) | Some(3) => Ok(true),
        Some(4) if unit.ends_with(".service") => {
            let unit = format!("{}.timer", unit.trim_end_matches(".service"));
            systemd_has_timer(&unit)
        }
        Some(4) => Ok(false),
        _ => cmd_assert_status(cmd).map(|_| false),
    }
}

/// Remove a systemd transient timer.
///
/// Errors if the timer is not available.
/// This may ask for root privileges through sudo.
///
/// Because this involves timers, if this operation fails it will also internally try to do the
/// same for the given unit with a `.timer` suffix.
pub fn systemd_remove_timer(unit: &str) -> Result<()> {
    // Invoke command, collect result
    let result = cmd_systemctl(&["--system", "--quiet", "stop", unit])
        .stderr(Stdio::null())
        .status()
        .map_err(Err::Systemctl);

    // Do the same with .timer unit suffix, ensure one succeeds
    if unit.ends_with(".service") {
        let unit = format!("{}.timer", unit.trim_end_matches(".service"));
        systemd_remove_timer(&unit).or_else(|_| cmd_assert_status(result?))
    } else {
        cmd_assert_status(result?)
    }
}

/// Invoke a systemd-run command with the given arguments.
fn systemd_run<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    cmd_assert_status(cmd_systemd_run(args).status().map_err(Err::SystemdRun)?)
}

// /// Invoke a systemctl command with the given arguments.
// fn systemctl<I, S>(args: I) -> Result<()>
// where
//     I: IntoIterator<Item = S>,
//     S: AsRef<OsStr>,
// {
//     cmd_assert_status(cmd_systemctl(args).status().map_err(Err::Systemctl)?)
// }

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

/// Build a systemctl command to run.
fn cmd_systemctl<I, S>(args: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(SUDO_BIN);
    cmd.arg("--");
    cmd.arg(SYSTEMCTL_BIN);
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
    #[error("failed to invoke systemd-run command")]
    SystemdRun(#[source] std::io::Error),

    #[error("failed to invoke systemctl command")]
    Systemctl(#[source] std::io::Error),

    #[error("systemd exited with non-zero status code: {0}")]
    Status(std::process::ExitStatus),
}
