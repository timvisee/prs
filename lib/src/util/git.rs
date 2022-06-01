use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use crate::git;

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

/// A whitelist of SSH hosts that support connection persisting.
const SSH_PERSIST_HOST_WHITELIST: [&str; 2] = ["github.com", "gitlab.com"];

lazy_static! {
    /// Cache for SSH connection persistence support guess.
    static ref SSH_PERSIST_GUESS_CACHE: Mutex<HashMap<PathBuf, bool>> = Mutex::new(HashMap::new());
}

/// Configure given git command to use SSH connection persisting.
///
/// `guess_ssh_connection_persist_support` should be used to guess whether this is supported.
pub fn configure_ssh_persist(cmd: &mut Command) {
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
pub fn guess_ssh_persist_support(repo: &Path) -> bool {
    // We must be using Unix, unreliable on Windows (and others?)
    if !cfg!(unix) {
        return false;
    }

    // User must not have set GIT_SSH_COMMAND variable
    if env::var_os(GIT_ENV_SSH).is_some() {
        return false;
    }

    // Get cached result
    if let Ok(guard) = (*SSH_PERSIST_GUESS_CACHE).lock() {
        if let Some(supported) = guard.get(repo) {
            return *supported;
        }
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
    if uri.len() >= 3 {
        Some(uri)
    } else {
        None
    }
}
