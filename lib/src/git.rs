use std::path::Path;
use std::process::{Command, Output};
use std::time::SystemTime;

use anyhow::Result;
use thiserror::Error;

/// The git FETCH_HEAD file.
const GIT_FETCH_HEAD_FILE: &str = ".git/FETCH_HEAD";

/// Invoke git pull.
pub fn git_pull(repo: &Path) -> Result<()> {
    git(repo, "pull -q")
}

/// Check if repository has remote configured.
pub fn git_has_remote(repo: &Path) -> Result<bool> {
    let output = git_output(repo, "remote")?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok(!stdout.trim().is_empty())
}

/// Get the current git branch name.
pub fn git_current_branch(repo: &Path) -> Result<String> {
    let output = git_output(repo, "rev-parse --abbrev-ref HEAD")?;
    let branch = std::str::from_utf8(&output.stdout)?.trim();
    assert!(!branch.is_empty(), "git returned invalid branch name");
    Ok(branch.into())
}

/// Get upstream branch for given branch if there is any.
pub fn git_branch_upstream<S: AsRef<str>>(repo: &Path, reference: S) -> Result<Option<String>> {
    let output = git_output(
        repo,
        format!("rev-parse --abbrev-ref {}@{{upstream}}", reference.as_ref()),
    )?;
    let upstream = std::str::from_utf8(&output.stdout)?.trim();
    if upstream.is_empty() {
        return Ok(None);
    }
    assert!(
        upstream.contains("/"),
        "git returned invalid upstream branch name"
    );
    Ok(Some(upstream.into()))
}

/// Get the hash of a reference.
pub fn git_ref_hash<S: AsRef<str>>(repo: &Path, reference: S) -> Result<String> {
    let output = git_output(repo, format!("rev-parse {}", reference.as_ref()))?;
    let hash = std::str::from_utf8(&output.stdout)?.trim();
    assert_eq!(hash.len(), 40, "git returned invalid hash");
    Ok(hash.into())
}

/// Get system time the repository was last pulled.
/// See: https://stackoverflow.com/a/9229377/1000145 (stat -c %Y .git/FETCH_HEAD)
pub fn git_last_pull_time(repo: &Path) -> Result<SystemTime> {
    Ok(repo
        .join(GIT_FETCH_HEAD_FILE)
        .metadata()
        .map_err(Err::Other)?
        .modified()
        .map_err(Err::Other)?)
}

/// Invoke a git command.
fn git<S: AsRef<str>>(repo: &Path, cmd: S) -> Result<()> {
    system(
        format!("git -C {:?} {}", repo.display(), cmd.as_ref()),
        Some(&repo),
    )
    .map_err(|err| Err::GitCli(err).into())
}

/// Invoke a git command.
fn git_output<S: AsRef<str>>(repo: &Path, cmd: S) -> Result<Output> {
    system_output(
        format!("git -C {:?} {}", repo.display(), cmd.as_ref()),
        Some(&repo),
    )
    .map_err(|err| Err::GitCli(err).into())
}

/// Invoke the given system command.
// TODO: do not create two subprocesses here (sh & git)
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
// TODO: do not create two subprocesses here (sh & git)
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
    Other(#[source] std::io::Error),

    #[error("failed to complete git operation")]
    GitCli(#[source] anyhow::Error),

    #[error("failed to invoke system command")]
    System(#[source] std::io::Error),

    #[error("system command exited with non-zero status code: {0}")]
    Status(std::process::ExitStatus),
}
