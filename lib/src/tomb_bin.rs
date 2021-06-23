use std::env;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, ExitStatus};

use anyhow::Result;
use thiserror::Error;

/// Binary name.
pub const BIN_NAME: &str = "tomb";

// /// Invoke tomb dig.
// pub fn tomb_dig(tomb_file: &Path) -> Result<()> {
//     unimplemented!();
//     // See: https://github.com/roddhjav/pass-tomb/blob/241964e227f373307354bc764c4ffab4326604ea/tomb.bash#L305-L312
//     tomb(&[
//         // TODO: "-q",
//         "dig",
//     ])
// }

/// Invoke tomb open.
pub fn tomb_open(tomb_file: &Path, key_file: &Path, store_dir: &Path) -> Result<()> {
    // TODO: ensure tomb file, key and store dir exist

    // TODO: do not set -q flag if in verbose mode?
    tomb(&[
        "-q",
        "open",
        tomb_file
            .to_str()
            .expect("tomb path contains invalid UTF-8"),
        "-k",
        key_file.to_str().expect("tomb path contains invalid UTF-8"),
        "-g",
        store_dir
            .to_str()
            .expect("password store directory path contains invalid UTF-8"),
    ])
}

/// Invoke tomb close.
pub fn tomb_close(tomb_file: &Path) -> Result<()> {
    // TODO: do not set -q flag if in verbose mode?
    tomb(&[
        "-q",
        "close",
        name(tomb_file).expect("failed to get tomb name"),
    ])
}

// /// Invoke tomb resize.
// pub fn tomb_resize(tomb_file: &Path, key_file: &Path, size_mb: u32) -> Result<()> {
//     // TODO: ensure tomb file and key exist, size must be larger

//     // TODO: do not set -q flag if in verbose mode?
//     tomb(&[
//         // TODO: "-q",
//         "resize",
//         tomb_file
//             .to_str()
//             .expect("tomb path contains invalid UTF-8"),
//         "-k",
//         key_file.to_str().expect("tomb path contains invalid UTF-8"),
//         "-s",
//         &format!("{}", size_mb),
//     ])
// }

/// Get tomb name based on path.
pub fn name(path: &Path) -> Option<&str> {
    path.file_name()?.to_str()?.rsplitn(2, ".").last()
}

/// Invoke a tomb command with the given arguments.
///
/// The command will take over the user console for in/output.
fn tomb<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    cmd_assert_status(cmd_tomb(args).status().map_err(Err::Tomb)?)
}

/// Build a tomb command to run.
fn cmd_tomb<I, S>(args: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = if let Ok(bin) = env::var("PASSWORD_STORE_TOMB") {
        Command::new(bin)
    } else {
        Command::new(BIN_NAME)
    };
    cmd.arg("-f");
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
    #[error("failed to invoke tomb command")]
    Tomb(#[source] std::io::Error),

    #[error("tomb operation exited with non-zero status code: {0}")]
    Status(std::process::ExitStatus),
}
