use std::thread;
use std::time::Duration;

use anyhow::Result;
use copypasta_ext::prelude::*;
#[cfg(all(feature = "notify", target_os = "linux", not(target_env = "musl")))]
use notify_rust::Hint;
#[cfg(all(feature = "notify", not(target_env = "musl")))]
use notify_rust::Notification;
use prs_lib::types::Plaintext;
use thiserror::Error;

use crate::util::error::{self, ErrorHintsBuilder};

/// Copy the given plain text to the user clipboard.
pub fn copy(data: &[u8]) -> Result<()> {
    let mut ctx = copypasta_ext::x11_fork::ClipboardContext::new().map_err(Err::Clipboard)?;
    ctx.set_contents(std::str::from_utf8(data).unwrap().into())
        .map_err(|err| Err::Clipboard(err).into())
}

/// Copy the given plain text to the user clipboard.
#[allow(unreachable_code)]
pub fn copy_timeout(data: &[u8], timeout: u64, report: bool) -> Result<()> {
    if timeout == 0 {
        return copy(data);
    }

    // macOS
    #[cfg(target_os = "macos")]
    {
        return copy_timeout_macos(data, timeout, report);
    }

    // X11 with musl
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "android", target_os = "emscripten")),
        target_env = "musl",
    ))]
    {
        return copy_timeout_x11_bin(data, timeout, report);
    }

    // X11
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "android", target_os = "emscripten")),
        not(target_env = "musl"),
    ))]
    {
        return copy_timeout_x11(data, timeout, report);
    }

    // Other clipboard contexts
    copy_timeout_fallback(data, timeout, report)
}

/// Copy with timeout on X11.
///
/// Keeps clipboard contents in clipboard even if application quits. Doesn't fuck with other
/// clipboard contents and reverts back to previous contents once a timeout is reached.
///
/// Forks & detaches two processes to set/keep clipboard contents and to drive the timeout.
///
/// Based on: https://docs.rs/copypasta-ext/0.3.2/copypasta_ext/x11_fork/index.html
// TODO: add support for Wayland on Linux as well
#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten")),
    not(target_env = "musl")
))]
fn copy_timeout_x11(data: &[u8], timeout: u64, report: bool) -> Result<()> {
    use copypasta_ext::{
        copypasta::x11_clipboard::{Clipboard, Selection},
        x11_fork::{ClipboardContext, Error},
    };
    use x11_clipboard::Clipboard as X11Clipboard;

    // Remember previous clipboard contents
    let mut ctx = ClipboardContext::new().map_err(Err::Clipboard)?;
    let previous = ctx.get_contents().unwrap_or_else(|_| String::new());

    let bin = crate::util::bin_name();

    // Detach fork to set given clipboard contents, keeps in clipboard until changed
    let setter_pid = match unsafe { libc::fork() } {
        -1 => return Err(Error::Fork.into()),
        0 => {
            // Obtain new X11 clipboard context, set clipboard contents
            let clip = X11Clipboard::new()
                .expect(&format!("{}: failed to obtain X11 clipboard context", bin,));
            clip.store(
                Clipboard::atom(&clip.setter.atoms),
                clip.setter.atoms.utf8_string,
                data,
            )
            .expect(&format!(
                "{}: failed to set clipboard contents through forked process",
                bin,
            ));

            // Wait for clipboard to change, then kill fork
            clip.load_wait(
                Clipboard::atom(&clip.getter.atoms),
                clip.getter.atoms.utf8_string,
                clip.getter.atoms.property,
            )
            .expect(&format!(
                "{}: failed to wait on new clipboard value in forked process",
                bin,
            ));

            // Update cleared state, show notification
            let _ = notify_cleared();

            error::quit();
        }
        pid => pid,
    };

    // Detach fork to revert clipboard after timeout unless changed
    match unsafe { libc::fork() } {
        -1 => return Err(Error::Fork.into()),
        0 => {
            thread::sleep(Duration::from_secs(timeout));

            // Determine if clipboard is already cleared, which is the case if the fork that set
            // the clipboard has died
            let cleared = unsafe {
                let pid_search_status = libc::kill(setter_pid, 0);
                let errno = *libc::__errno_location() as i32;
                pid_search_status == -1 && errno == libc::ESRCH
            };

            // Revert to previous clipboard contents if not yet cleared
            if !cleared {
                let mut ctx = ClipboardContext::new()
                    .expect(&format!("{}: failed to obtain X11 clipboard context", bin,));
                ctx.set_contents(previous).expect(&format!(
                    "{}: failed to revert clipboard contents through forked process",
                    bin,
                ));
            }

            error::quit();
        }
        _pid => {}
    }

    if report {
        eprintln!(
            "Secret copied to clipboard. Waiting {} seconds to clear...",
            timeout
        );
    }

    Ok(())
}

/// Copy with timeout on X11 using xclip or xsel binaries.
///
/// Keeps clipboard contents in clipboard even if application quits. Doesn't fuck with other
/// clipboard contents and reverts back to previous contents once a timeout is reached.
///
/// Forks & detaches two processes to set/keep clipboard contents and to drive the timeout.
///
/// Based on: https://docs.rs/copypasta-ext/0.3.2/copypasta_ext/x11_fork/index.html
#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten")),
    target_env = "musl",
))]
fn copy_timeout_x11_bin(data: &[u8], timeout: u64, report: bool) -> Result<()> {
    use copypasta_ext::x11_bin::X11BinClipboardContext as ClipboardContext;

    let data = std::str::from_utf8(data).map_err(Err::Utf8)?;
    let bin = crate::util::bin_name();

    // Remember previous clipboard contents
    let mut ctx = ClipboardContext::new().map_err(Err::Clipboard)?;
    let previous = ctx.get_contents().unwrap_or_else(|_| String::new());

    // Set clipboard
    ctx.set_contents(data.to_string()).map_err(Err::Clipboard)?;

    // Detach fork to revert clipboard after timeout unless changed
    match unsafe { libc::fork() } {
        -1 => panic!("failed to fork"),
        0 => {
            thread::sleep(Duration::from_secs(timeout));

            // Obtain new clipboard context, get current contents
            let mut ctx = ClipboardContext::new()
                .expect(&format!("{}: failed to obtain X11 clipboard context", bin,));
            let now = ctx.get_contents().expect(&format!(
                "{}: failed to get clipboard contents through forked process",
                bin,
            ));

            // If clipboard contents didn't change, revert back to previous
            if data == now {
                ctx.set_contents(previous).expect(&format!(
                    "{}: failed to revert clipboard contents through forked process",
                    bin,
                ));

                // Update cleared state, show notification
                let _ = notify_cleared();
            }

            error::quit();
        }
        _pid => {}
    }

    if report {
        eprintln!(
            "Secret copied to clipboard. Waiting {} seconds to clear...",
            timeout
        );
    }

    Ok(())
}

/// Copy with timeout on macOS.
///
/// Keeps clipboard contents in clipboard even if application quits. Doesn't fuck with other
/// clipboard contents and reverts back to previous contents once a timeout is reached.
///
/// Forks & detaches two processes to set/keep clipboard contents and to drive the timeout.
///
/// Based on: https://docs.rs/copypasta-ext/0.3.2/copypasta_ext/x11_fork/index.html
#[cfg(target_os = "macos")]
fn copy_timeout_macos(data: &[u8], timeout: u64, report: bool) -> Result<()> {
    use copypasta_ext::copypasta::ClipboardContext;

    let data = std::str::from_utf8(data).map_err(Err::Utf8)?;
    let bin = crate::util::bin_name();

    // Remember previous clipboard contents
    let mut ctx = ClipboardContext::new().map_err(Err::Clipboard)?;
    let previous = ctx.get_contents().unwrap_or_else(|_| String::new());

    // Set clipboard
    ctx.set_contents(data.to_string()).map_err(Err::Clipboard)?;

    // Detach fork to revert clipboard after timeout unless changed
    match unsafe { libc::fork() } {
        -1 => panic!("failed to fork"),
        0 => {
            thread::sleep(Duration::from_secs(timeout));

            // Obtain new clipboard context, get current contents
            let mut ctx = ClipboardContext::new()
                .expect(&format!("{}: failed to obtain X11 clipboard context", bin,));
            let now = ctx.get_contents().expect(&format!(
                "{}: failed to get clipboard contents through forked process",
                bin,
            ));

            // If clipboard contents didn't change, revert back to previous
            if data == now {
                ctx.set_contents(previous).expect(&format!(
                    "{}: failed to revert clipboard contents through forked process",
                    bin,
                ));

                // Update cleared state, show notification
                let _ = notify_cleared();
            }

            error::quit();
        }
        _pid => {}
    }

    if report {
        eprintln!(
            "Secret copied to clipboard. Waiting {} seconds to clear...",
            timeout
        );
    }

    Ok(())
}

/// Copy with timeout.
///
/// Simple fallback method using delay in console.
fn copy_timeout_fallback(data: &[u8], timeout: u64, report: bool) -> Result<()> {
    // TODO: do not use x11 context here!
    use copypasta_ext::x11_fork::ClipboardContext;

    let mut ctx = ClipboardContext::new().map_err(Err::Clipboard)?;
    ctx.set_contents(std::str::from_utf8(data).unwrap().into())
        .map_err(Err::Clipboard)?;

    // TODO: clear clipboard on ctrl+c
    if report {
        eprintln!(
            "Secret copied to clipboard. Waiting {} seconds to clear...",
            timeout
        );
    }
    thread::sleep(Duration::from_secs(timeout));

    ctx.set_contents("".into()).map_err(Err::Clipboard)?;
    let _ = notify_cleared();

    Ok(())
}

/// Copy the given plain text to the user clipboard.
pub(crate) fn plaintext_copy(
    mut plaintext: Plaintext,
    first_line: bool,
    error_empty: bool,
    report: bool,
    timeout: u64,
) -> Result<()> {
    if first_line {
        plaintext = plaintext.first_line()?;
    }

    // Do not copy empty secret
    if error_empty && plaintext.is_empty() {
        error::quit_error_msg(
            "secret is empty, did not copy to clipboard",
            ErrorHintsBuilder::default().force(true).build().unwrap(),
        )
    }

    copy_timeout(plaintext.unsecure_ref(), timeout, report).map_err(Err::CopySecret)?;

    Ok(())
}

/// Show notification to user about cleared clipboard.
#[allow(unreachable_code)]
pub(crate) fn notify_cleared() -> Result<()> {
    // Do not show notification with not notify or on musl due to segfault
    #[cfg(all(feature = "notify", not(target_env = "musl")))]
    {
        let mut n = Notification::new();
        n.appname(&crate::util::bin_name())
            .summary(&format!("Clipboard cleared - {}", crate::util::bin_name()))
            .body("Secret cleared from clipboard")
            .auto_icon()
            .icon("lock")
            .timeout(3000);

        #[cfg(target_os = "linux")]
        n.urgency(notify_rust::Urgency::Low)
            .hint(Hint::Category("presence.offline".into()));

        n.show()?;
        return Ok(());
    }

    // Fallback if we cannot notify
    eprintln!("Secret cleared from clipboard");
    Ok(())
}

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum Err {
    #[error("failed to parse clipboard contents as UTF-8")]
    Utf8(#[source] std::str::Utf8Error),

    #[error("failed to set clipboard")]
    Clipboard(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("failed to copy secret to clipboard")]
    CopySecret(#[source] anyhow::Error),
}
