use std::env;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, ExitStatus, Output};
use std::time::SystemTime;

use anyhow::Result;
use thiserror::Error;

/// The git FETCH_HEAD file.
const GIT_FETCH_HEAD_FILE: &str = ".git/FETCH_HEAD";

/// Environment variable git uses to modify the ssh command.
#[cfg(not(windows))]
const GIT_ENV_SSH: &str = "GIT_SSH_COMMAND";

/// Custom ssh command for git.
///
/// With this custom SSH command we enable SSH session reuse, to make remote git operations much
/// quicker for repositories using an SSH URL. This greatly improves prs sync speeds.
// TODO: make configurable, add current user ID to path
#[cfg(not(windows))]
const GIT_ENV_SSH_CMD: &str = "ssh -o 'ControlMaster auto' -o 'ControlPath /tmp/.prs-session--%r@%h:%p' -o 'ControlPersist 1h'";

/// Invoke git init.
pub fn git_init(repo: &Path) -> Result<()> {
    git(repo, &["init", "-q"])
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
    git(repo, &args)
}

/// Git stage all files and changes.
pub fn git_add_all(repo: &Path) -> Result<()> {
    git(repo, &["add", "."])
}

/// Invoke git commit.
pub fn git_commit(repo: &Path, msg: &str, commit_empty: bool) -> Result<()> {
    // Quit if no changes and we don't allow empty commit
    if !commit_empty && !git_has_changes(repo)? {
        return Ok(());
    }

    // TODO: add -q
    let mut args = vec!["commit", "-q", "--no-edit", "-m", msg];
    if commit_empty {
        args.push("--allow-empty");
    }
    git(repo, &args)
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
    git(repo, &args)
}

/// Invoke git pull.
pub fn git_pull(repo: &Path) -> Result<()> {
    // TODO: do not set -q flag if in verbose mode?
    git(repo, &["pull", "-q"])
}

/// Invoke git fetch.
pub fn git_fetch(repo: &Path, reference: Option<&str>) -> Result<()> {
    // TODO: do not set -q flag if in verbose mode?
    let mut args = vec!["fetch", "-q"];
    if let Some(reference) = reference {
        args.push(reference);
    }
    git(repo, &args)
}

/// Check if repository has (staged/unstaged) changes.
pub fn git_has_changes(repo: &Path) -> Result<bool> {
    Ok(!git_stdout_ok(repo, &["status", "-s"])?.is_empty())
}

/// Check if repository has remote configured.
pub fn git_has_remote(repo: &Path) -> Result<bool> {
    Ok(!git_stdout_ok(repo, &["remote"])?.is_empty())
}

/// Git get remote list.
pub fn git_remote(repo: &Path) -> Result<Vec<String>> {
    Ok(git_stdout_ok(repo, &["remote"])?
        .lines()
        .map(|r| r.into())
        .collect())
}

/// Get get remote URL.
pub fn git_remote_get_url(repo: &Path, remote: &str) -> Result<String> {
    Ok(git_stdout_ok(repo, &["remote", "get-url", remote])?)
}

/// Get add remote URL.
pub fn git_remote_add(repo: &Path, remote: &str, url: &str) -> Result<()> {
    Ok(git(repo, &["remote", "add", remote, url])?)
}

/// Get remove remote URL.
pub fn git_remote_remove(repo: &Path, remote: &str) -> Result<()> {
    Ok(git(repo, &["remote", "remove", remote])?)
}

/// Get the current git branch name.
pub fn git_current_branch(repo: &Path) -> Result<String> {
    let branch = git_stdout_ok(repo, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    assert!(!branch.is_empty(), "git returned empty branch name");
    assert!(!branch.contains("\n"), "git returned multiple branches");
    Ok(branch.into())
}

/// List remote git branches.
pub fn git_branch_remote(repo: &Path) -> Result<Vec<String>> {
    Ok(git_stdout_ok(repo, &["branch", "-r", "--no-color"])?
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
        upstream.contains("/"),
        "git returned invalid upstream branch name"
    );
    Ok(Some(upstream.into()))
}

/// Set upstream branch for the given branch.
pub fn git_branch_set_upstream(repo: &Path, reference: Option<&str>, upstream: &str) -> Result<()> {
    let mut args = vec!["branch", "--set-upstream-to", &upstream];
    if let Some(reference) = reference {
        args.push(reference);
    }
    git(repo, &args)
}

/// Get the hash of a reference.
pub fn git_ref_hash<S: AsRef<str>>(repo: &Path, reference: S) -> Result<String> {
    let hash = git_stdout_ok(repo, &["rev-parse", reference.as_ref()])?;
    assert_eq!(hash.len(), 40, "git returned invalid hash");
    Ok(hash.into())
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

/// Get git repository state.
///
/// See:
/// - https://github.com/libgit2/libgit2/blob/52294c413100ed4930764addc69beadd82382a4c/src/repository.c#L2867-L2908
/// - https://libgit2.org/libgit2/#HEAD/type/git_repository_state_t
pub fn git_state(repo: &Path) -> Result<RepoState> {
    let git_dir = repo.join(".git");
    let git_dir = if git_dir.is_dir() { &git_dir } else { repo };

    if git_dir.join("rebase-merge/interactive").is_file() {
        Ok(RepoState::RebaseInteractive)
    } else if git_dir.join("rebase-merge").is_dir() {
        Ok(RepoState::RebaseMerge)
    } else if git_dir.join("rebase-apply/rebasing").is_file() {
        Ok(RepoState::Rebase)
    } else if git_dir.join("rebase-apply/applying").is_file() {
        Ok(RepoState::ApplyMailbox)
    } else if git_dir.join("rebase-apply").is_dir() {
        Ok(RepoState::ApplyMailboxOrRebase)
    } else if git_dir.join("MERGE_HEAD").is_file() {
        Ok(RepoState::Merge)
    } else if git_dir.join("REVERT_HEAD").is_file() {
        if git_dir.join("sequencer/todo").is_file() {
            Ok(RepoState::RevertSequence)
        } else {
            Ok(RepoState::Revert)
        }
    } else if git_dir.join("CHERRY_PICK_HEAD").is_file() {
        if git_dir.join("sequencer/todo").is_file() {
            Ok(RepoState::CherryPickSequence)
        } else {
            Ok(RepoState::CherryPick)
        }
    } else if git_dir.join("BISECT_LOG").is_file() {
        Ok(RepoState::Bisect)
    } else {
        Ok(RepoState::Clean)
    }
}

/// Git repository state.
///
/// See:
/// - https://libgit2.org/libgit2/#HEAD/type/git_repository_state_t
/// - https://libgit2.org/libgit2/#HEAD/group/repository/git_repository_state
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum RepoState {
    Clean,
    Merge,
    Revert,
    RevertSequence,
    CherryPick,
    CherryPickSequence,
    Bisect,
    Rebase,
    RebaseInteractive,
    RebaseMerge,
    ApplyMailbox,
    ApplyMailboxOrRebase,
}

/// Invoke a git command with the given arguments.
///
/// The command will take over the user console for in/output.
fn git<I, S>(repo: &Path, args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    cmd_assert_status(cmd_git(args, Some(repo)).status().map_err(Err::System)?)
}

/// Invoke a git command, returns output.
fn git_output<I, S>(repo: &Path, args: I) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    cmd_git(args, Some(repo))
        .output()
        .map_err(|err| Err::System(err).into())
}

/// Invoke a git command with the given arguments, return stdout on success.
fn git_stdout_ok<I, S>(repo: &Path, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = git_output(repo, args)?;
    cmd_assert_status(output.status)?;

    Ok(std::str::from_utf8(&output.stdout)
        .map_err(|err| Err::GitCli(err.into()))?
        .trim()
        .into())
}

/// Build a git command to run.
fn cmd_git<I, S>(args: I, dir: Option<&Path>) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new("git");

    if let Some(dir) = dir {
        cmd.arg("-C");
        cmd.arg(dir);
        cmd.current_dir(dir);
    }

    // Set custom git ssh command to speed up remote operations
    #[cfg(not(windows))]
    {
        if env::var_os(GIT_ENV_SSH).is_none() {
            cmd.env(GIT_ENV_SSH, GIT_ENV_SSH_CMD);
        }
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

    #[error("system command exited with non-zero status code: {0}")]
    Status(std::process::ExitStatus),
}
