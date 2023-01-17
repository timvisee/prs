pub mod cli;
#[cfg(feature = "clipboard")]
pub mod clipboard;
pub mod edit;
pub mod error;
pub mod fs;
pub mod pass;
pub mod progress;
pub mod secret;
pub mod select;
pub mod select_basic;
#[cfg(feature = "select-fzf-bin")]
pub mod select_fzf_bin;
#[cfg(all(feature = "select-skim", unix))]
pub mod select_skim;
#[cfg(feature = "select-skim-bin")]
pub mod select_skim_bin;
pub mod stdin;
pub mod style;
pub mod sync;
#[cfg(all(feature = "tomb", target_os = "linux"))]
pub mod time;
#[cfg(all(feature = "tomb", target_os = "linux"))]
pub mod tomb;
#[cfg(feature = "totp")]
pub mod totp;

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::util::error::{quit_error_msg, ErrorHints};

/// Invoke a command.
///
/// Quit on error.
// TODO: do not wrap commands in sh/cmd, we should not have to do this and only causes problems
// TODO: provide list of arguments instead of a command string for better reliability/compatability
pub fn invoke_cmd(cmd: String, dir: Option<&Path>, verbose: bool) -> Result<(), std::io::Error> {
    if verbose {
        eprintln!("Invoking: {}\n", cmd);
    }

    // Invoke command
    let mut process = Command::new(if cfg!(not(windows)) { "sh" } else { "cmd" });
    process.arg(if cfg!(not(windows)) { "-c" } else { "/C" });
    process.arg(&cmd);
    if let Some(dir) = dir {
        process.current_dir(dir);
    }
    let status = process.status()?;

    // Report status errors
    if !status.success() {
        eprintln!();
        quit_error_msg(
            format!(
                "{} exited with status code {}",
                cmd.trim_start().split(' ').next().unwrap_or("command"),
                status.code().unwrap_or(-1)
            ),
            ErrorHints::default(),
        );
    }

    Ok(())
}

/// Get the name of the executable that was invoked.
///
/// When a symbolic or hard link is used, the name of the link is returned.
///
/// This attempts to obtain the binary name in the following order:
/// - name in first item of program arguments via `std::env::args`
/// - current executable name via `std::env::current_exe`
/// - crate name
pub fn bin_name() -> String {
    env::args_os()
        .next()
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .or_else(|| env::current_exe().ok())
        .and_then(|p| p.file_name().map(|n| n.to_owned()))
        .and_then(|n| n.into_string().ok())
        .unwrap_or_else(|| crate::NAME.into())
}
