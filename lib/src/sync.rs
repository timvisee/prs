//! Password store synchronization functionality.

use std::path::Path;
use std::time::Duration;

use anyhow::Result;

use crate::{
    git::{self, RepositoryState},
    Store,
};

/// Store git directory.
pub const STORE_GIT_DIR: &str = ".git/";

/// Duration after which pull refs are considered outdated.
///
/// If the last pull is within this duration, some operations such as a push may be optimized away
/// if not needed.
pub const GIT_PULL_OUTDATED: Duration = Duration::from_secs(30);

/// Sync helper for given store.
pub struct Sync<'a> {
    /// The store.
    store: &'a Store,
}

impl<'a> Sync<'a> {
    /// Construct new sync helper for given store.
    pub fn new(store: &'a Store) -> Sync<'a> {
        Self { store }
    }

    /// Get the repository path.
    fn path(&self) -> &Path {
        &self.store.root
    }

    /// Check readyness of store for syncing.
    ///
    /// This checks whether the repository state is clean, which means that there's no active
    /// merge/rebase/etc.
    /// The repository might be dirty, use `sync_is_dirty` to check that.
    pub fn readyness(&self) -> Result<Readyness> {
        let path = self.path();

        if !self.is_init() {
            return Ok(Readyness::NoSync);
        }

        match git::git_state(path).unwrap() {
            RepositoryState::Clean => {
                if is_dirty(path)? {
                    Ok(Readyness::Dirty)
                } else {
                    Ok(Readyness::Ready)
                }
            }
            state => Ok(Readyness::RepoState(state)),
        }
    }

    /// Prepare the store for new changes.
    ///
    /// - If sync is not initialized, it does nothing.
    /// - If sync remote is set, it pulls changes.
    pub fn prepare(&self) -> Result<()> {
        // TODO: return error if dirty?

        // Skip if no sync
        if !self.is_init() {
            return Ok(());
        }

        // We're done if we don't have a remote
        if !self.has_remote()? {
            return Ok(());
        }

        // We must have upstream set, otherwise try to automatically set or don't pull
        let repo = self.path();
        if git::git_branch_upstream(repo, "HEAD")?.is_none() {
            // Get remotes, we cannot decide upstream if we don't have exactly one
            let remotes = git::git_remote(repo)?;
            if remotes.len() != 1 {
                return Ok(());
            }

            // Fetch remote branches
            let remote = &remotes[0];
            git::git_fetch(repo, Some(remote))?;

            // List remote branches, stop if there are none
            let remote_branches = git::git_branch_remote(repo)?;
            if remote_branches.is_empty() {
                return Ok(());
            }

            // Determine upstream reference
            let branch = git::git_current_branch(repo)?;
            let upstream_ref = format!("{}/{}", remote, branch);

            // Set upstream reference if available on remote, otherwise stop
            if !remote_branches.contains(&upstream_ref) {
                return Ok(());
            }
            git::git_branch_set_upstream(repo, None, &upstream_ref)?;
        }

        self.pull()?;

        Ok(())
    }

    /// Finalize the store with new changes.
    ///
    /// - If sync is not initialized, it does nothing.
    /// - If sync is initialized, it commits changes.
    /// - If sync remote is set, it pushes changes.
    pub fn finalize<M: AsRef<str>>(&self, msg: M) -> Result<()> {
        // Skip if no sync
        if !self.is_init() {
            return Ok(());
        }

        // Commit changes if dirty
        if is_dirty(self.path())? {
            self.commit_all(msg, false)?;
        }

        // Do not push  if no remote or not out of sync
        if !self.has_remote()? || !safe_need_to_push(self.path()) {
            return Ok(());
        }

        // We must have upstream set, otherwise try to automatically set or don't push
        let mut set_branch = None;
        let mut set_upstream = None;
        let repo = self.path();
        if git::git_branch_upstream(repo, "HEAD")?.is_none() {
            // Get remotes, we cannot decide upstream if we don't have exactly one
            let remotes = git::git_remote(repo)?;
            if remotes.len() == 1 {
                // Fetch and list remote branches
                let remote = &remotes[0];
                git::git_fetch(repo, Some(remote))?;
                let remote_branches = git::git_branch_remote(repo)?;

                // Determine upstream reference
                let branch = git::git_current_branch(repo)?;
                let upstream_ref = format!("{}/{}", remote, branch);

                // Set upstream reference if not yet used on remote
                if !remote_branches.contains(&upstream_ref) {
                    set_branch = Some(branch);
                    set_upstream = Some(remote.to_string());
                }
            }
        }

        self.push(set_branch.as_deref(), set_upstream.as_deref())?;

        Ok(())
    }

    /// Initialize sync.
    pub fn init(&self) -> Result<()> {
        git::git_init(self.path())?;
        self.commit_all("Initialize sync with git", true)?;
        Ok(())
    }

    /// Clone sync from a remote URL.
    pub fn clone(&self, url: &str, quiet: bool) -> Result<()> {
        let path = self
            .path()
            .to_str()
            .expect("failed to determine clone path");
        git::git_clone(self.path(), url, path, quiet)?;
        Ok(())
    }

    /// Check whether sync has been initialized in this store.
    pub fn is_init(&self) -> bool {
        self.path().join(STORE_GIT_DIR).is_dir()
    }

    /// Get a list of sync remotes.
    pub fn remotes(&self) -> Result<Vec<String>> {
        git::git_remote(self.path())
    }

    /// Get the URL of the given remote.
    pub fn remote_url(&self, remote: &str) -> Result<String> {
        git::git_remote_get_url(self.path(), remote)
    }

    /// Add the URL of the given remote.
    pub fn add_remote_url(&self, remote: &str, url: &str) -> Result<()> {
        git::git_remote_add(self.path(), remote, url)
    }

    /// Set the URL of the given remote.
    pub fn set_remote_url(&self, remote: &str, url: &str) -> Result<()> {
        // Do not set but remove and add to flush any fetched remote data
        git::git_remote_remove(self.path(), remote)?;
        self.add_remote_url(remote, url)
    }

    /// Check whether this store has a remote configured.
    pub fn has_remote(&self) -> Result<bool> {
        if !self.is_init() {
            return Ok(false);
        }
        git::git_has_remote(self.path())
    }

    /// Pull changes from remote.
    fn pull(&self) -> Result<()> {
        git::git_pull(self.path())
    }

    /// Push changes to remote.
    fn push(&self, set_branch: Option<&str>, set_upstream: Option<&str>) -> Result<()> {
        git::git_push(self.path(), set_branch, set_upstream)
    }

    /// Add all changes and commit them.
    pub fn commit_all<M: AsRef<str>>(&self, msg: M, commit_empty: bool) -> Result<()> {
        let path = self.path();
        git::git_add_all(path)?;
        git::git_commit(path, msg.as_ref(), commit_empty)
    }

    /// Hard reset all changes.
    pub fn reset_hard_all(&self) -> Result<()> {
        let path = self.path();
        git::git_add_all(path)?;
        git::git_reset_hard(path)
    }
}

/// Defines readyness of store sync.
///
/// Some states block sync usage, including:
/// - Sync not initialized
/// - Git repository is dirty
#[derive(Debug, Eq, PartialEq)]
pub enum Readyness {
    /// Sync is not initialized for this store.
    NoSync,

    /// Special repository state.
    RepoState(git::RepositoryState),

    /// Repository is dirty (has uncommitted changes).
    Dirty,

    /// Ready to sync.
    Ready,
}

impl Readyness {
    /// Check if ready.
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }
}

/// Check if repository is dirty.
///
/// Repository is dirty if it has any uncommitted changed.
fn is_dirty(repo: &Path) -> Result<bool> {
    git::git_has_changes(repo)
}

/// Check whether we need to push to the remote.
///
/// This defaults to true on error.
fn safe_need_to_push(repo: &Path) -> bool {
    match need_to_push(repo) {
        Ok(push) => push,
        Err(err) => {
            eprintln!(
                "failed to test if local branch is different than remote, ignoring: {}",
                err,
            );
            true
        }
    }
}

/// Check whether we need to push to the remote.
///
/// If the upstream branch is unknown, this always returns true.
fn need_to_push(repo: &Path) -> Result<bool> {
    // If last pull is outdated, always push
    let last_pulled = git::git_last_pull_time(repo)?;
    if last_pulled.elapsed()? > GIT_PULL_OUTDATED {
        return Ok(true);
    }

    // Get branch and upstream branch name
    let branch = git::git_current_branch(repo)?;
    let upstream = match git::git_branch_upstream(repo, &branch)? {
        Some(upstream) => upstream,
        None => return Ok(true),
    };

    // Compare local and remote branch hashes
    Ok(git::git_ref_hash(repo, branch)? != git::git_ref_hash(repo, upstream)?)
}
