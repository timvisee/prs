use std::path::Path;
use std::process::{Command, Output};

use anyhow::Result;
use thiserror::Error;

/// Invoke git pull.
pub fn git_pull(repository: &Path) -> Result<()> {
    git(repository, "pull -q")
}

/// Check if repository has remote configured.
pub fn git_has_remote(repository: &Path) -> Result<bool> {
    let output = git_output(repository, "remote")?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok(!stdout.trim().is_empty())
}

/// Invoke a git command.
fn git<S: AsRef<str>>(repository: &Path, cmd: S) -> Result<()> {
    system(
        format!("git -C {:?} {}", repository.display(), cmd.as_ref()),
        Some(&repository),
    )
    .map_err(|err| Err::GitCli(err).into())
}

/// Invoke a git command.
fn git_output<S: AsRef<str>>(repository: &Path, cmd: S) -> Result<Output> {
    system_output(
        format!("git -C {:?} {}", repository.display(), cmd.as_ref()),
        Some(&repository),
    )
    .map_err(|err| Err::GitCli(err).into())
}

/// Invoke the given system command.
fn system(cmd: String, dir: Option<&Path>) -> Result<()> {
    // Invoke command
    // TODO: make this compatible with Windows
    let mut process = Command::new("sh");
    process.arg("-c").arg(&cmd);
    if let Some(dir) = dir {
        process.current_dir(dir);
    }
    let status = process.status().map_err(Err::System)?;

    // Report status errors
    if !status.success() {
        return Err(Err::Status(status).into());
    }

    Ok(())
}

/// Invoke the given system command.
fn system_output(cmd: String, dir: Option<&Path>) -> Result<Output> {
    // Invoke command
    // TODO: make this compatible with Windows
    let mut process = Command::new("sh");
    process.arg("-c").arg(&cmd);
    if let Some(dir) = dir {
        process.current_dir(dir);
    }
    let output = process.output().map_err(Err::System)?;

    // Report status errors
    if !output.status.success() {
        return Err(Err::Status(output.status).into());
    }

    Ok(output)
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to complete git operation")]
    GitCli(#[source] anyhow::Error),

    #[error("failed to invoke system command")]
    System(#[source] std::io::Error),

    #[error("system command exited with non-zero status code: {0}")]
    Status(std::process::ExitStatus),
}
