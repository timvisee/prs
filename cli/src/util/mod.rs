pub mod cli;
pub mod clipboard;
pub mod error;
pub mod skim;
pub mod stdin;
pub mod style;
pub mod sync;

use std::path::Path;
use std::process::Command;

use crate::util::error::{quit_error_msg, ErrorHints};

/// Invoke a command.
///
/// Quit on error.
pub fn invoke_cmd(cmd: String, dir: Option<&Path>, verbose: bool) -> Result<(), std::io::Error> {
    if verbose {
        eprintln!("Invoking: {}\n", cmd);
    }

    // Invoke command
    // TODO: make this compatible with Windows
    let mut process = Command::new("sh");
    process.arg("-c").arg(&cmd);
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
                cmd.trim_start().split(" ").next().unwrap_or("command"),
                status.code().unwrap_or(-1)
            ),
            ErrorHints::default(),
        );
    }

    Ok(())
}
