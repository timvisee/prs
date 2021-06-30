use std::path::{Path, PathBuf};

/// Maximum symlink resolving depth.
const SYMLINK_DEPTH_MAX: u8 = 31;

/// Local TTY path.
const LOCAL_TTY_PATH: &str = "/dev/stdin";

/// Get TTY path for this process.
pub fn get_tty() -> PathBuf {
    let path = PathBuf::from(LOCAL_TTY_PATH);
    resolve_symlink(&path, 0)
}

/// Resolve symlink to the final accessible path.
///
/// # Panics
///
/// Panics if a depth of `SYMLINK_DEPTH_MAX` is reached to prevent infinite loops.
fn resolve_symlink(path: &Path, depth: u8) -> PathBuf {
    // Panic if we're getting too deep
    if depth >= SYMLINK_DEPTH_MAX {
        // TODO: do not panic, return last unique path or return error
        panic!("failed to resolve symlink because it is too deep, possible loop?");
    }

    // Read symlink path, recursively find target
    match path.read_link() {
        Ok(path) => resolve_symlink(&path, depth + 1),
        Err(_) => path.into(),
    }
}
