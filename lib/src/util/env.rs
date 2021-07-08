use std::env;

/// Check whether we're in a Wayland environment
pub fn is_wayland() -> bool {
    has_non_empty_env("WAYLAND_DISPLAY")
}

/// Check whether `GPG_TTY` is set.
pub fn has_gpg_tty() -> bool {
    has_non_empty_env("GPG_TTY")
}

/// Check if an environment variable is set and is not empty.
pub fn has_non_empty_env(env: &str) -> bool {
    env::var_os(env).map(|v| !v.is_empty()).unwrap_or(false)
}
