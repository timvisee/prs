use std::collections::HashMap;
use std::env;
#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

#[cfg(unix)]
use crate::Store;
use crate::git;

#[cfg(unix)]
use ofiles::opath;

/// Environment variable git uses to modify the ssh command.
const GIT_ENV_SSH: &str = "GIT_SSH_COMMAND";

/// Custom ssh command for git.
///
/// With this custom SSH command we enable SSH connection persistence for session reuse to make
/// remote git operations much quicker for repositories using an SSH URL. This greatly improves prs
/// sync speeds.
///
/// This sets up a session file in the users `/tmp` directory. A timeout of 10 seconds is set to
/// quickly abort a connection attempt if the persistent connection fails.
const SSH_PERSIST_CMD: &str = "ssh -o 'ControlMaster auto' -o 'ControlPath /tmp/.prs-session--%r@%h:%p' -o 'ControlPersist 1h' -o 'ConnectTimeout 10'";

/// Directory for SSH persistent session files.
#[cfg(unix)]
pub(crate) const SSH_PERSIST_SESSION_FILE_DIR: &str = "/tmp";

/// Prefix for SSH persistent session files.
#[cfg(unix)]
pub(crate) const SSH_PERSIST_SESSION_FILE_PREFIX: &str = ".prs-session--";

/// A whitelist of SSH hosts that support connection persisting.
const SSH_PERSIST_HOST_WHITELIST: [&str; 2] = ["github.com", "gitlab.com"];

lazy_static! {
    /// Cache for SSH connection persistence support guess.
    static ref SSH_PERSIST_GUESS_CACHE: Mutex<HashMap<PathBuf, bool>> = Mutex::new(HashMap::new());
}

/// Configure given git command to use SSH connection persisting.
///
/// `guess_ssh_connection_persist_support` should be used to guess whether this is supported.
pub(crate) fn configure_ssh_persist(cmd: &mut Command) {
    cmd.env(self::GIT_ENV_SSH, self::SSH_PERSIST_CMD);
}

/// Guess whether SSH connection persistence is supported.
///
/// This does a best effort to determine whether SSH connection persistence is supported. This is
/// used to enable connection reuse. This internally caches the guess in the current process by
/// repository path.
///
/// - Disabled on non-Unix
/// - Disabled if user set `GIT_SSH_COMMAND`
/// - Requires all repository SSH remote hosts to be whitelisted
///
/// Related: https://gitlab.com/timvisee/prs/-/issues/31
/// Related: https://github.com/timvisee/prs/issues/5#issuecomment-803940880
// TODO: make configurable, add current user ID to path
pub(crate) fn guess_ssh_persist_support(repo: &Path) -> bool {
    // We must be using Unix, unreliable on Windows (and others?)
    if !cfg!(unix) {
        return false;
    }

    // User must not have set GIT_SSH_COMMAND variable
    if env::var_os(GIT_ENV_SSH).is_some() {
        return false;
    }

    // Get cached result
    if let Ok(guard) = (*SSH_PERSIST_GUESS_CACHE).lock()
        && let Some(supported) = guard.get(repo)
    {
        return *supported;
    }

    // Gather git remotes, assume not supported if no remote or error
    let remotes = match git::git_remote(repo) {
        Ok(remotes) if remotes.is_empty() => return false,
        Ok(remotes) => remotes,
        Err(_) => return false,
    };

    // Get remote host bits, ensure we have all
    let ssh_uris: Vec<_> = remotes
        .iter()
        .filter_map(|remote| git::git_remote_get_url(repo, remote).ok())
        .filter(|uri| !remote_is_http(uri))
        .collect();

    // Ensure all SSH URI hosts are part of whitelist, assume incompatible on error
    let supported = ssh_uris.iter().all(|uri| match ssh_uri_host(uri) {
        Some(host) => SSH_PERSIST_HOST_WHITELIST.contains(&host.to_lowercase().as_str()),
        None => false,
    });

    // Cache result
    if let Ok(mut guard) = (*SSH_PERSIST_GUESS_CACHE).lock() {
        guard.insert(repo.to_path_buf(), supported);
    }

    supported
}

/// Check if given git remote URI is using HTTP(S) rather than SSH.
fn remote_is_http(mut url: &str) -> bool {
    url = url.trim();
    url.starts_with("http://") || url.starts_with("https://")
}

/// Grab the host bit of an SSH URI.
///
/// This will do a best effort to grap the host bit of an SSH URI. If an HTTP(S) URL is given, or
/// if the host bit could not be determined, `None` is returned. Note that this may not be very
/// reliable.
#[allow(clippy::manual_split_once, clippy::needless_splitn)]
fn ssh_uri_host(mut uri: &str) -> Option<&str> {
    // Must not be a HTTP(S) URL
    if remote_is_http(uri) {
        return None;
    }

    // Strip any ssh prefix
    if let Some(stripped) = uri.strip_prefix("ssh://") {
        uri = stripped;
    }

    // Strip the URI until we're left with the host
    // TODO: this is potentially unreliable, improve this logic
    let before_slash = uri.splitn(2, '/').next().unwrap();
    let after_at = before_slash.splitn(2, '@').last().unwrap();
    let before_collon = after_at.splitn(2, ':').next().unwrap();
    let uri = before_collon.trim();

    // Ensure the host is at least 3 characters long
    if uri.len() >= 3 { Some(uri) } else { None }
}

/// Kill SSH clients that have an opened persistent session on a password store.
///
/// Closing these is required to close any open Tomb mount.
#[cfg(unix)]
pub fn kill_ssh_by_session(store: &Store) {
    // If persistent SSH isn't used, we don't have to close sessions
    if !guess_ssh_persist_support(&store.root) {
        return;
    }

    // TODO: guess SSH session directory and file details from environment variable

    // Find prs persistent SSH session files
    let dir = match fs::read_dir(SSH_PERSIST_SESSION_FILE_DIR) {
        Ok(dir) => dir,
        Err(_) => return,
    };
    let session_files = dir
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_socket()).unwrap_or(false))
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|n| n.starts_with(SSH_PERSIST_SESSION_FILE_PREFIX))
                .unwrap_or(false)
        })
        .map(|e| e.path());

    // For each session file, kill attached SSH clients
    session_files.for_each(|p| {
        // List PIDs having this session file open
        let pids = match opath(p) {
            Ok(pids) => pids,
            Err(_) => return,
        };

        pids.into_iter()
            .map(Into::into)
            .filter(|pid: &u32| pid > &0 && pid < &(i32::MAX as u32))
            .filter(|pid| {
                // Only handle ssh clients
                fs::read_to_string(format!("/proc/{pid}/cmdline"))
                    .map(|cmdline| {
                        let cmd = cmdline.split([' ', ':']).next().unwrap();
                        cmd.starts_with("ssh")
                    })
                    .unwrap_or(true)
            })
            .for_each(|pid| {
                if let Err(err) = nix::sys::signal::kill(
                    nix::unistd::Pid::from_raw(pid as i32),
                    Some(nix::sys::signal::Signal::SIGTERM),
                ) {
                    eprintln!("Failed to kill persistent SSH client (pid: {pid}): {err}",);
                }
            });
    });
}
