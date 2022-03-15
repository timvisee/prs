use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, ExitStatus, Output};
use std::time::SystemTime;

use anyhow::Result;
use thiserror::Error;

use crate::util;

// Re-exports
pub use git_state::{git_state, RepositoryState};

/// Binary name.
#[cfg(not(windows))]
pub const BIN_NAME: &str = "git";
#[cfg(windows)]
pub const BIN_NAME: &str = "git.exe";

/// The git FETCH_HEAD file.
const GIT_FETCH_HEAD_FILE: &str = ".git/FETCH_HEAD";

/// Invoke git init.
pub fn git_init(repo: &Path) -> Result<()> {
    git(repo, &["init", "-q"], false)
}

/// Invoke git clone.
///
/// Shows progress unless `quiet` is set.
pub fn git_clone(repo: &Path, url: &str, path: &str, quiet: bool) -> Result<()> {
    let mut args = vec!["clone", "-q"];
    if !quiet {
        args.push("--progress");
    }
    args.extend_from_slice(&[url, path]);
    git(repo, &args, true)
}

/// Git stage all files and changes.
pub fn git_add_all(repo: &Path) -> Result<()> {
    git(repo, &["add", "."], false)
}

/// Invoke git commit.
pub fn git_commit(repo: &Path, msg: &str, commit_empty: bool) -> Result<()> {
    // Quit if no changes and we don't allow empty commit
    if !commit_empty && !git_has_changes(repo)? {
        return Ok(());
    }

    let mut args = vec!["commit", "-q", "--no-edit", "-m", msg];
    if commit_empty {
        args.push("--allow-empty");
    }
    git(repo, &args, false)
}

/// Invoke git push.
pub fn git_push(repo: &Path, set_branch: Option<&str>, set_upstream: Option<&str>) -> Result<()> {
    // TODO: do not set -q flag if in verbose mode?
    let mut args = vec!["push", "-q"];
    if let Some(upstream) = set_upstream {
        args.extend_from_slice(&["--set-upstream", upstream]);
    }
    if let Some(branch) = set_branch {
        args.push(branch);
    }
    git(repo, &args, true)
}

/// Invoke git pull.
pub fn git_pull(repo: &Path) -> Result<()> {
    // TODO: do not set -q flag if in verbose mode?
    git(repo, &["pull", "-q"], true)
}

/// Invoke git fetch.
pub fn git_fetch(repo: &Path, reference: Option<&str>) -> Result<()> {
    // TODO: do not set -q flag if in verbose mode?
    let mut args = vec!["fetch", "-q"];
    if let Some(reference) = reference {
        args.push(reference);
    }
    git(repo, &args, true)
}

/// Check if repository has (staged/unstaged) changes.
pub fn git_has_changes(repo: &Path) -> Result<bool> {
    Ok(!git_stdout_ok(repo, &["status", "-s"], false)?.is_empty())
}

/// Check if repository has remote configured.
pub fn git_has_remote(repo: &Path) -> Result<bool> {
    Ok(!git_stdout_ok(repo, &["remote"], false)?.is_empty())
}

/// Git get remote list.
pub fn git_remote(repo: &Path) -> Result<Vec<String>> {
    Ok(git_stdout_ok(repo, &["remote"], false)?
        .lines()
        .map(|r| r.into())
        .collect())
}

/// Get get remote URL.
pub fn git_remote_get_url(repo: &Path, remote: &str) -> Result<String> {
    git_stdout_ok(repo, &["remote", "get-url", remote], false)
}

/// Get add remote URL.
pub fn git_remote_add(repo: &Path, remote: &str, url: &str) -> Result<()> {
    git(repo, &["remote", "add", remote, url], false)
}

/// Get remove remote URL.
pub fn git_remote_remove(repo: &Path, remote: &str) -> Result<()> {
    git(repo, &["remote", "remove", remote], false)
}

/// Get the current git branch name.
pub fn git_current_branch(repo: &Path) -> Result<String> {
    let branch = git_stdout_ok(repo, &["rev-parse", "--abbrev-ref", "HEAD"], false)?;
    assert!(!branch.is_empty(), "git returned empty branch name");
    assert!(!branch.contains('\n'), "git returned multiple branches");
    Ok(branch)
}

/// List remote git branches.
pub fn git_branch_remote(repo: &Path) -> Result<Vec<String>> {
    Ok(git_stdout_ok(repo, &["branch", "-r", "--no-color"], false)?
        .lines()
        .map(|r| {
            match r.strip_prefix("* ") {
                Some(r) => r,
                None => r,
            }
            .to_string()
        })
        .collect())
}

/// Get upstream branch for given branch if there is any.
///
/// If there is none, `None` is returned.
pub fn git_branch_upstream<S: AsRef<str>>(repo: &Path, reference: S) -> Result<Option<String>> {
    // Invoke command
    let output = git_output(
        repo,
        &[
            "rev-parse",
            "--abbrev-ref",
            &format!("{}@{{upstream}}", reference.as_ref()),
        ],
        false,
    )?;

    // Scan stderr for 'no upstream' messages
    let stderr = std::str::from_utf8(&output.stderr)
        .map_err(|err| Err::GitCli(err.into()))?
        .trim();
    if stderr.contains("fatal: no upstream configured for branch") {
        return Ok(None);
    }

    // Assert status
    cmd_assert_status(output.status)?;

    // Find upstream branch
    let upstream = std::str::from_utf8(&output.stdout)
        .map_err(|err| Err::GitCli(err.into()))?
        .trim();
    if upstream.is_empty() {
        return Ok(None);
    }
    assert!(
        upstream.contains('/'),
        "git returned invalid upstream branch name"
    );
    Ok(Some(upstream.into()))
}

/// Set upstream branch for the given branch.
pub fn git_branch_set_upstream(repo: &Path, reference: Option<&str>, upstream: &str) -> Result<()> {
    let mut args = vec!["branch", "--set-upstream-to", upstream];
    if let Some(reference) = reference {
        args.push(reference);
    }
    git(repo, &args, false)
}

/// Get the hash of a reference.
pub fn git_ref_hash<S: AsRef<str>>(repo: &Path, reference: S) -> Result<String> {
    let hash = git_stdout_ok(repo, &["rev-parse", reference.as_ref()], false)?;
    assert_eq!(hash.len(), 40, "git returned invalid hash");
    Ok(hash)
}

/// Get system time the repository was last pulled.
/// See: https://stackoverflow.com/a/9229377/1000145 (stat -c %Y .git/FETCH_HEAD)
pub fn git_last_pull_time(repo: &Path) -> Result<SystemTime> {
    Ok(repo
        .join(GIT_FETCH_HEAD_FILE)
        .metadata()
        .and_then(|m| m.modified())
        .map_err(Err::Other)?)
}

/// Invoke a git command with the given arguments.
///
/// The command will take over the user console for in/output.
fn git<I, S>(repo: &Path, args: I, connects_remote: bool) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    cmd_assert_status(
        cmd_git(args, repo, connects_remote)
            .status()
            .map_err(Err::System)?,
    )
}

/// Invoke a git command, returns output.
fn git_output<I, S>(repo: &Path, args: I, connects_remote: bool) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    cmd_git(args, repo, connects_remote)
        .output()
        .map_err(|err| Err::System(err).into())
}

/// Invoke a git command with the given arguments, return stdout on success.
fn git_stdout_ok<I, S>(repo: &Path, args: I, connects_remote: bool) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = git_output(repo, args, connects_remote)?;
    cmd_assert_status(output.status)?;

    Ok(std::str::from_utf8(&output.stdout)
        .map_err(|err| Err::GitCli(err.into()))?
        .trim()
        .into())
}

/// Build a git command to run.
fn cmd_git<I, S>(args: I, dir: &Path, connects_remote: bool) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(BIN_NAME);
    cmd.arg("-C");
    cmd.arg(dir);
    cmd.current_dir(dir);

    // Configure session reuse if connecting to a remote and supported
    if connects_remote && util::git::guess_ssh_persist_support(dir) {
        util::git::configure_ssh_persist(&mut cmd);
    }

    cmd.args(args);

    // Debug invoked git commands
    // eprintln!("Invoked: {:?}", &cmd);

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
    #[error("failed to complete git operation")]
    Other(#[source] std::io::Error),

    #[error("failed to complete git operation")]
    GitCli(#[source] anyhow::Error),

    #[error("failed to invoke system command")]
    System(#[source] std::io::Error),

    #[error("git operation exited with non-zero status code: {0}")]
    Status(std::process::ExitStatus),
}
