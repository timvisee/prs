use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use thiserror::Error;

use crate::store::Store;

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

        match git2::Repository::open(path).map_err(Err::Git2)?.state() {
            git2::RepositoryState::Clean => {
                if is_dirty(path)? {
                    Ok(Readyness::Dirty)
                } else {
                    Ok(Readyness::Ready)
                }
            }
            state => Ok(Readyness::GitState(state)),
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

        // Pull if remote
        if self.has_remote()? {
            self.pull()?;
        }

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

        // Push if remote and out of sync
        if self.has_remote()? && safe_need_to_push(self.path()) {
            self.push()?;
        }

        Ok(())
    }

    /// Initialize sync.
    pub fn init(&self) -> Result<()> {
        crate::git::git_init(self.path())?;
        self.commit_all("Initialize sync with git", true)?;
        Ok(())
    }

    /// Check whether sync has been initialized in this store.
    // TODO: make private?
    pub fn is_init(&self) -> bool {
        self.path().join(STORE_GIT_DIR).is_dir()
    }

    /// Get a list of sync remotes.
    pub fn remotes(&self) -> Result<Vec<String>> {
        crate::git::git_remote(self.path())
    }

    /// Get the URL of the given remote.
    pub fn remote_url(&self, remote: &str) -> Result<String> {
        crate::git::git_remote_get_url(self.path(), remote)
    }

    /// Add the URL of the given remote.
    pub fn add_remote_url(&self, remote: &str, url: &str) -> Result<()> {
        crate::git::git_remote_add_url(self.path(), remote, url)
    }

    /// Set the URL of the given remote.
    pub fn set_remote_url(&self, remote: &str, url: &str) -> Result<()> {
        crate::git::git_remote_set_url(self.path(), remote, url)
    }

    /// Check whether this store has a remote configured.
    pub fn has_remote(&self) -> Result<bool> {
        if !self.is_init() {
            return Ok(false);
        }
        crate::git::git_has_remote(self.path())
    }

    /// Pull changes from remote.
    fn pull(&self) -> Result<()> {
        crate::git::git_pull(self.path())
    }

    /// Push changes to remote.
    fn push(&self) -> Result<()> {
        let repo = rustygit::Repository::new(self.path());
        repo.push().map_err(Err::RustyGit)?;
        Ok(())
    }

    /// Add all changes and commit them.
    fn commit_all<M: AsRef<str>>(&self, msg: M, commit_empty: bool) -> Result<()> {
        let path = self.path();
        crate::git::git_add_all(path)?;
        crate::git::git_commit(path, msg.as_ref(), commit_empty)
    }
}

/// Defines readyness of store sync.
///
/// Some states block sync usage, including:
/// - Sync not initialized
/// - Git repository is dirty
// TODO: add NoRemote state?
#[derive(Debug)]
pub enum Readyness {
    /// Sync is not initialized for this store.
    NoSync,

    /// Special repository state.
    GitState(git2::RepositoryState),

    /// Repository is dirty (has uncommitted changes).
    Dirty,

    /// Ready to sync.
    Ready,
}

impl Readyness {
    /// Check if ready.
    pub fn is_ready(&self) -> bool {
        match self {
            Self::Ready => true,
            _ => false,
        }
    }
}

/// Check if repository is dirty.
///
/// Repository is dirty if it has any uncommitted changed.
fn is_dirty(repo: &Path) -> Result<bool> {
    let repo = git2::Repository::open(repo).map_err(Err::Git2)?;
    let statuses = repo.statuses(None).map_err(Err::Git2)?;
    Ok(!statuses.is_empty())
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
    let last_pulled = crate::git::git_last_pull_time(repo)?;
    if last_pulled.elapsed()? > GIT_PULL_OUTDATED {
        return Ok(true);
    }

    // Get branch and upstream branch name
    let branch = crate::git::git_current_branch(repo)?;
    let upstream = match crate::git::git_branch_upstream(repo, &branch)? {
        Some(upstream) => upstream,
        None => return Ok(true),
    };

    // Compare local and remote branch hashes
    Ok(crate::git::git_ref_hash(repo, branch)? != crate::git::git_ref_hash(repo, upstream)?)
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to complete git operation")]
    Git2(#[source] git2::Error),

    #[error("failed to complete git operation")]
    RustyGit(#[source] rustygit::error::GitError),
}
