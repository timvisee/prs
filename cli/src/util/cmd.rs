use std::ffi::OsStr;
use std::process::Command;

/// Get the current command to spawn a subprocess.
#[cfg(feature = "clipboard")]
pub(crate) fn current_cmd() -> Option<Command> {
    let current_exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(_) => match std::env::args().next() {
            Some(bin) => bin.into(),
            None => return None,
        },
    };

    Some(Command::new(current_exe))
}

/// Command extensions.
pub(crate) trait CommandExt {
    fn arg_if<S: AsRef<OsStr>>(&mut self, arg: S, condition: bool) -> &mut Command;
}

impl CommandExt for Command {
    fn arg_if<S: AsRef<OsStr>>(&mut self, arg: S, condition: bool) -> &mut Command {
        if condition {
            self.arg(arg)
        } else {
            self
        }
    }
}
