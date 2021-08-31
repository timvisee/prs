use std::path::{Path, PathBuf};

/// Maximum symlink resolving depth.
const SYMLINK_DEPTH_MAX: u8 = 31;

/// Local TTY path.
const LOCAL_TTY_PATH: &str = "/dev/stdin";

/// Get TTY path for this process.
///
/// Returns `None` if not in a TTY. Always returns `None` if not Linux, FreeBSD or OpenBSD.
pub fn get_tty() -> Option<PathBuf> {
    // None on unsupported platforms
    if cfg!(not(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "openbsd",
    ))) {
        return None;
    }

    let path = PathBuf::from(LOCAL_TTY_PATH);
    resolve_symlink(&path, 0)
}

/// Resolve symlink to the final accessible path.
///
/// Returns `None` if the given link could not be read (and `depth` is 0).
///
/// # Panics
///
/// Panics if a depth of `SYMLINK_DEPTH_MAX` is reached to prevent infinite loops.
fn resolve_symlink(path: &Path, depth: u8) -> Option<PathBuf> {
    // Panic if we're getting too deep
    if depth >= SYMLINK_DEPTH_MAX {
        // TODO: do not panic, return last unique path or return error
        panic!("failed to resolve symlink because it is too deep, possible loop?");
    }

    // Read symlink path, recursively find target
    match path.read_link() {
        Ok(path) => resolve_symlink(&path, depth + 1),
        Err(_) if depth == 0 => None,
        Err(_) => Some(path.into()),
    }
}
