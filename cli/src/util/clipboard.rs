use std::io::{Error as IoError, ErrorKind as IoErrorKind, Write};
use std::process::{Child, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use base64::Engine;
use copypasta_ext::display::DisplayServer;
use copypasta_ext::prelude::*;
#[cfg(all(feature = "notify", target_os = "linux", not(target_env = "musl")))]
use notify_rust::Hint;
#[cfg(all(feature = "notify", not(target_env = "musl")))]
use notify_rust::Notification;
use prs_lib::Plaintext;
use thiserror::Error;

use crate::util::{
    cmd::{self, CommandExt},
    error::{self, ErrorHintsBuilder},
};

/// Delay for checking changed clipboard in `timeout_or_clip_change`.
const TIMEOUT_CLIP_CHECK_DELAY: Duration = Duration::from_secs(3);

/// Delay between each spin in `timeout_or_clip_change`.
const TIMEOUT_CLIP_SPIN_DELAY: Duration = Duration::from_secs(5);

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten")),
))]
/// Additional time in which a `TempClip` will be reused after its timeout has been reached.
const TEMP_CLIP_REUSE_PERIOD: Duration = Duration::from_secs(1);

lazy_static! {
    /// Shared clipboard context.
    static ref CLIP: SharedContext = Default::default();

    /// Shared clipboard manager in this process.
    static ref CLIP_MANAGER: ClipManager = Default::default();
}

/// Copy the given data to the clipboard.
pub(crate) fn copy(data: &Plaintext, quiet: bool, verbose: bool) -> Result<()> {
    CLIP_MANAGER.copy(data, quiet, verbose)
}

/// Copy the given data to the clipboard, revert clipboard state after timeout.
pub(crate) fn copy_timeout_revert(
    data: Plaintext,
    timeout: Duration,
    quiet: bool,
    verbose: bool,
) -> Result<()> {
    CLIP_MANAGER.copy_timeout_revert(data, timeout, quiet, verbose)
}

/// Copy the given plain text to the user clipboard.
pub(crate) fn copy_plaintext(
    mut plaintext: Plaintext,
    first_line: bool,
    error_empty: bool,
    quiet: bool,
    verbose: bool,
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

    // Copy with timeout
    copy_timeout_revert(plaintext, Duration::from_secs(timeout), quiet, verbose)
        .map_err(Err::CopySecret)?;
    Ok(())
}

/// Clipboard context shared across this process.
///
/// This is lazy and initializes the context on use.
#[derive(Default)]
struct SharedContext {
    ctx: Mutex<Option<Box<dyn ClipboardProviderExt>>>,
}

impl SharedContext {
    /// Ensure clipboard context is ready or error.
    ///
    /// If the context wasn't made ready yet, it will be initialized now.
    fn ensure_context(&self) -> Result<()> {
        let mut guard = self.ctx.lock().unwrap();
        match *guard {
            Some(_) => Ok(()),
            None => {
                *guard = Some(copypasta_ext::try_context().ok_or(Err::NoProvider)?);
                Ok(())
            }
        }
    }

    /// Get clipboard contents.
    pub fn get(&self) -> Result<Plaintext> {
        self.ensure_context()?;
        let mut guard = self.ctx.lock().unwrap();
        let ctx = guard.as_mut().ok_or(Err::NoProvider)?;

        Ok(ctx
            .get_contents()
            .map(|d| d.into_bytes().into())
            .unwrap_or_else(|_| Plaintext::empty()))
    }

    /// Set clipboard contents.
    pub fn set(&self, data: &Plaintext) -> Result<()> {
        self.ensure_context()?;
        let mut guard = self.ctx.lock().unwrap();
        let ctx = guard.as_mut().ok_or(Err::NoProvider)?;

        ctx.set_contents(data.unsecure_to_str().unwrap().into())
            .map_err(|e| Err::Set(anyhow!(e)))?;

        Ok(())
    }

    /// Get clipboard context display server.
    pub fn display_server(&self) -> Result<Option<DisplayServer>> {
        self.ensure_context()?;
        Ok(self
            .ctx
            .lock()
            .unwrap()
            .as_mut()
            .ok_or(Err::NoProvider)?
            .display_server())
    }

    /// Check whether this clipboard context only holds the clipboard the lifetime of this binary.
    pub fn has_bin_lifetime(&self) -> Result<bool> {
        self.ensure_context()?;
        Ok(self
            .ctx
            .lock()
            .unwrap()
            .as_mut()
            .ok_or(Err::NoProvider)?
            .has_bin_lifetime())
    }
}

/// A global clipboard manager for prs.
///
/// Globally instantiated at `CLIP_MANAGER`.
///
/// This allows to temporarily set clipboard contents after which the original state is reverted.
/// On X11 and Wayland systems this ensures the clipboard is kept alive even if the prs binary
/// exits.
#[derive(Default)]
struct ClipManager {
    clip: Mutex<Option<TempClip>>,
}

impl ClipManager {
    /// Copy the given data to the clipboard without reverting.
    pub fn copy(&self, data: &Plaintext, quiet: bool, verbose: bool) -> Result<()> {
        // Stop any current clipboard without reverting
        if let Some(clip) = self.clip.lock().unwrap().take() {
            clip.stop_no_revert();
        }

        set(data, true, false, quiet, verbose).map_err(Err::Set)?;

        if !quiet {
            eprintln!("Secret copied to clipboard");
        }

        Ok(())
    }

    /// Copy the given data to the clipboard, revert clipboard state after timeout.
    pub fn copy_timeout_revert(
        &self,
        data: Plaintext,
        timeout: Duration,
        quiet: bool,
        verbose: bool,
    ) -> Result<()> {
        let mut clip_guard = self.clip.lock().unwrap();

        if !quiet {
            eprintln!(
                "Secret copied to clipboard. Clearing after {} seconds...",
                timeout.as_secs(),
            );
        }

        // If the temporary clipboard is still active, we should replace it
        if let Some(clip) = &mut *clip_guard {
            if clip.is_active() {
                clip.replace(data, timeout, quiet, verbose)
                    .map_err(Err::ClipMan)?;
                return Ok(());
            }
        }

        // Create new clip session
        *clip_guard = Some(TempClip::new(data, timeout, quiet, verbose).map_err(Err::ClipMan)?);

        Ok(())
    }
}

/// Temporary clipboard content handler.
///
/// Temporarily set clipboard contents, revert to original clipboard state after specified timeout.
/// The clipboard remains intact if it is changed in the meanwhile.
///
/// This internally spawns a subprocess which is disowned to handle the timeout in a non-blocking
/// manner. Dropping this struct doesn't forget the subprocess if still active keeping the timeout
/// and clipboard reversal intact.
///
/// Warning: two TempClip instances should never be used at the same time, as this will introduce
/// unexpected results.
struct TempClip {
    data: Plaintext,
    old_data: Plaintext,
    process: Option<Child>,
    timeout_until: Instant,
}

impl TempClip {
    /// Construct a new clipboard session.
    ///
    /// Copy the given data for the given timeout, then revert the clipboard.
    /// This internally spawns a subprocess to handle the timeout and reversal.
    pub fn new(data: Plaintext, timeout: Duration, quiet: bool, verbose: bool) -> Result<Self> {
        let mut session = Self {
            data,
            old_data: get()?,
            process: None,
            timeout_until: Instant::now() + timeout,
        };
        session.spawn(timeout, quiet, verbose)?;
        Ok(session)
    }

    /// Replace the contents of this clipboard session and reset the timeout.
    ///
    /// This keeps the original clipboard state intact but respawns the timeout and reversal subprocess.
    pub fn replace(
        &mut self,
        data: Plaintext,
        timeout: Duration,
        quiet: bool,
        verbose: bool,
    ) -> Result<()> {
        self.data = data;
        self.timeout_until = Instant::now() + timeout;
        self.spawn(timeout, quiet, verbose)?;
        Ok(())
    }

    /// Spawn or respawn detached process to copy, timeout and revert.
    fn spawn(&mut self, timeout: Duration, quiet: bool, verbose: bool) -> Result<()> {
        // Kill current child to spawn new process
        self.kill_child();

        self.process = Some(spawn_process_copy_revert(
            &self.data,
            &self.old_data,
            timeout.as_secs(),
            quiet,
            verbose,
        )?);
        Ok(())
    }

    /// Check whether a reverting subprocess is currently active.
    fn is_active(&mut self) -> bool {
        // Assume active if child is still running
        let active = self.process.as_mut().map(is_child_running).unwrap_or(false);

        // On X11/Wayland systems the child remains active until the reverted clipboard is
        // replaced, this means that the lifetime may be a lot longer than the clipboard timeout.
        // On these systems, if active, we must test against the timeout time as well. If the
        // timeout is reached with a small grace period, this isn't active anymore.
        #[cfg(all(
            unix,
            not(any(target_os = "macos", target_os = "android", target_os = "emscripten")),
        ))]
        if active && (self.timeout_until + TEMP_CLIP_REUSE_PERIOD) <= Instant::now() {
            return false;
        }

        active
    }

    /// Kill the subprocess if still alive.
    fn kill_child(&mut self) {
        let mut process = self.process.take();

        // Child must still be running
        if !process.as_mut().map(is_child_running).unwrap_or(false) {
            return;
        }

        // Detach and kill child
        if let Some(mut child) = process {
            match child.kill() {
                Ok(_) => {}
                Err(err) if err.kind() == IoErrorKind::InvalidInput => {}
                Err(err) => {
                    error::print_warning(format!(
                        "failed to kill clipboard subprocess, may cause weird behavior: {}",
                        err
                    ));
                }
            }
        }
    }

    /// Stop this temporary clipboard without reverting.
    pub fn stop_no_revert(mut self) {
        self.kill_child();
    }

    /// Stop and revert the clipboard to its old contents if unchanged.
    ///
    /// If the clipboard data is still set, it reverts to the original clipboard state. If that
    /// state is unknown, the clipboard is cleared to prevent it holding sensitive data.
    #[allow(unused)]
    pub fn stop_revert(mut self) -> Result<()> {
        self.kill_child();

        // Get current contents, revert to old if failed
        let current = match get() {
            Ok(data) => data,
            Err(_) => {
                return set(&self.old_data, true, false, true, false);
            }
        };

        // If current data is still copied, revert to original state or clear for security reasons
        if current == self.data {
            if !self.old_data.is_empty() {
                return set(&self.old_data, true, false, true, false);
            } else {
                return set(&Plaintext::empty(), false, false, true, false);
            }
        }

        // If current data is empty, revert to old content if that isn't empty
        if current.is_empty() && !self.old_data.is_empty() {
            return set(&self.old_data, true, false, true, false);
        }

        Ok(())
    }
}

/// Get current clipboard contents.
fn get() -> Result<Plaintext> {
    CLIP.get()
}

/// Set clipboard data.
///
/// On X11/Wayland this only sets the clipboard for the lifetime of the current process.
/// Use `forever` to keep clipboard contents forever.
/// If `forever_blocks` is true, setting forever will block until the clipboard is changed. If
/// false, this spawns a subprocess to keep the clipboard non-blocking.
///
/// On other display servers this will always set the clipboard forever.
fn set(
    data: &Plaintext,
    forever: bool,
    forever_blocks: bool,
    quiet: bool,
    verbose: bool,
) -> Result<()> {
    // Set clipboard forever using subprocess/blocking on X11/Wayland if this context is limited to binary
    // lifetime
    if forever && CLIP.has_bin_lifetime().unwrap_or(false) {
        #[cfg(all(
            unix,
            not(any(target_os = "macos", target_os = "android", target_os = "emscripten")),
            not(target_env = "musl")
        ))]
        match CLIP.display_server() {
            Ok(Some(DisplayServer::X11)) if forever_blocks => {
                x11_set_blocking(data).map_err(Err::Set)?;
                return Ok(());
            }
            Ok(Some(DisplayServer::Wayland)) if forever_blocks => {
                set_blocking(data, quiet, verbose).map_err(Err::Set)?;
                return Ok(());
            }
            _ => {}
        }

        return spawn_process_copy(data, quiet, verbose).map(|_| ());
    }

    // Set clipboard normally
    CLIP.set(data)
}

/// On any clipboard system, set the clipboard and block until it is changed.
///
/// Warning: this implementation is very inefficient, and may block for up to a minute longer when
/// the clipboard has changed.
#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten")),
    not(target_env = "musl")
))]
fn set_blocking(data: &Plaintext, quiet: bool, verbose: bool) -> Result<()> {
    const BASE: Duration = Duration::from_secs(2);
    const MAX_DELAY: Duration = Duration::from_secs(60);

    // Set clipboard
    set(data, false, false, quiet, verbose)?;

    // Spin until changed
    let mut delay = BASE;
    loop {
        thread::sleep(delay);

        // Stop if changed
        if !CLIP.get().map(|d| &d == data).unwrap_or(false) {
            return Ok(());
        }

        // Increas edelay
        delay = (delay + BASE).min(MAX_DELAY);
    }
}

/// On X11 set the clipboard and block until its changed.
#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten")),
    not(target_env = "musl")
))]
fn x11_set_blocking(data: &Plaintext) -> Result<()> {
    use x11_clipboard::Clipboard as X11Clipboard;

    // Obtain new X11 clipboard context, set clipboard contents
    let clip = X11Clipboard::new().expect("failed to obtain X11 clipboard context");
    clip.store(
        clip.setter.atoms.clipboard,
        clip.setter.atoms.utf8_string,
        data.unsecure_ref(),
    )
    .map_err(|err| Err::Set(anyhow!(err)))?;

    // Wait for clipboard to change, then kill fork
    if let Err(err) = clip.load_wait(
        clip.setter.atoms.clipboard,
        clip.getter.atoms.utf8_string,
        clip.getter.atoms.property,
    ) {
        error::print_warning(format!(
            "failed wait for X11 clipboard change, may cause weird behavior: {}",
            err
        ));
    }

    Ok(())
}

/// Copy the given data to the clipboard in a subprocess.
fn spawn_process_copy(data: &Plaintext, quiet: bool, verbose: bool) -> Result<Child> {
    // Spawn & disown background process to set clipboard
    let mut process = cmd::current_cmd()
        .ok_or(Err::NoSubProcess)?
        .arg_if("--quiet", quiet)
        .arg_if("--verbose", verbose)
        .args(["internal", "clip"])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(Err::SpawnProcess)?;

    // Send data to copy to process
    writeln!(
        process.stdin.as_mut().unwrap(),
        "{}",
        base64::engine::general_purpose::STANDARD.encode(data.unsecure_ref()),
    )
    .map_err(Err::ConfigProcess)?;

    Ok(process)
}

/// Copy the given data to the clipboard in a subprocess.
/// Revert to the old data after the given timeout.
fn spawn_process_copy_revert(
    data: &Plaintext,
    data_old: &Plaintext,
    timeout_sec: u64,
    quiet: bool,
    verbose: bool,
) -> Result<Child> {
    // Spawn & disown background process to set clipboard
    let mut process = cmd::current_cmd()
        .ok_or(Err::NoSubProcess)?
        .arg_if("--quiet", quiet)
        .arg_if("--verbose", verbose)
        .args(["internal", "clip-revert"])
        .arg("--timeout")
        .arg(&format!("{}", timeout_sec))
        .stdin(Stdio::piped())
        .spawn()
        .map_err(Err::SpawnProcess)?;

    // Send data to copy to process
    writeln!(
        process.stdin.as_mut().unwrap(),
        "{},{}",
        base64::engine::general_purpose::STANDARD.encode(data.unsecure_ref()),
        base64::engine::general_purpose::STANDARD.encode(data_old.unsecure_ref()),
    )
    .map_err(Err::ConfigProcess)?;

    Ok(process)
}

/// Subprocess logic for `spawn_process_copy`.
///
/// This should be called in the subprocess that is spawned with `spawn_process_copy`.
///
/// Copies the given data to the clipboard.
pub(crate) fn subprocess_copy(data: &Plaintext, quiet: bool, verbose: bool) -> Result<()> {
    set(data, false, true, quiet, verbose).map_err(Err::Set)?;
    Ok(())
}

/// Subprocess logic for `spawn_process_copy_revert`.
///
/// This should be called in the subprocess that is spawned with `spawn_process_copy_revert`.
///
/// Copies the given data to the clipboard, and reverts to the old data after the timeout if the
/// clipboard contents have not been changed.
pub(crate) fn subprocess_copy_revert(
    data: &Plaintext,
    data_old: &Plaintext,
    timeout: Duration,
    quiet: bool,
    verbose: bool,
) -> Result<()> {
    set(data, false, false, quiet, verbose).map_err(Err::Set)?;

    // Wait for timeout or until clipboard is changed
    let changed = timeout_or_clip_change(data, timeout);

    // Revert clipboard to previous if contents didn't change
    if changed {
        if !quiet {
            notify_cleared(true, false).map_err(Err::Notify)?;
        }
    } else if &get().map_err(Err::Get)? == data {
        if !quiet {
            notify_cleared(false, !data_old.is_empty()).map_err(Err::Notify)?;
        }
        set(data_old, true, true, quiet, verbose).map_err(Err::Revert)?;
    }

    Ok(())
}

/// Wait for the given timeout or until the clipboard content is different than `data`.
///
/// Has a warmup of at least 3 seconds before it tests the clipboard for changes.
/// This is required to give the subprocess enough time to start and set the initial clipboard.
///
/// Returns `true` if returned early due to a clipboard change.
pub(crate) fn timeout_or_clip_change(data: &Plaintext, timeout: Duration) -> bool {
    let until = Instant::now() + timeout;
    let check_clip_from = Instant::now() + TIMEOUT_CLIP_CHECK_DELAY;

    loop {
        let now = Instant::now();

        // Return early if content has changed
        if now >= check_clip_from {
            let got = CLIP.get();
            if matches!(got, Ok(ref d) if d != data) {
                return true;
            }
        }

        // Test if timeout is reached
        if now >= until {
            return false;
        }

        // Wait a little before checking again
        thread::sleep((until - Instant::now()).min(TIMEOUT_CLIP_SPIN_DELAY));
    }
}

/// Show notification to user about cleared clipboard.
pub(crate) fn notify_cleared(changed: bool, restored: bool) -> Result<()> {
    // Do not show notification with not notify or on musl due to segfault
    #[cfg(all(feature = "notify", not(target_env = "musl")))]
    {
        let title = if changed {
            "changed"
        } else if restored {
            "restored"
        } else {
            "cleared"
        };
        let mut n = Notification::new();
        n.appname(&crate::util::bin_name())
            .summary(&format!(
                "Clipboard {} - {}",
                title,
                crate::util::bin_name()
            ))
            .body("Secret wiped from clipboard")
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
    #[allow(unreachable_code)]
    {
        eprintln!("Secret wiped from clipboard");
        Ok(())
    }
}

/// Check if the given child is still running.
///
/// Assumes no on failure.
fn is_child_running(child: &mut Child) -> bool {
    matches!(child.try_wait(), Ok(None))
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("no supported clipboard provider available")]
    NoProvider,

    #[error("failed to prepare prs clipboard manager")]
    ClipMan(#[source] anyhow::Error),

    #[error("failed to use clipboard, no way to spawn subprocess for clipboard manager, must run as standalone binary")]
    NoSubProcess,

    #[error("failed to spawn subprocess for clipboard manager")]
    SpawnProcess(#[source] IoError),

    #[error("failed to configure subprocess for clipboard manager")]
    ConfigProcess(#[source] IoError),

    #[error("failed to get clipboard contents")]
    Get(#[source] anyhow::Error),

    #[error("failed to set clipboard contents")]
    Set(#[source] anyhow::Error),

    #[error("failed to revert clipboard contents")]
    Revert(#[source] anyhow::Error),

    #[error("failed to copy secret to clipboard")]
    CopySecret(#[source] anyhow::Error),

    #[error("failed to notify user for cleared clipboard")]
    Notify(#[source] anyhow::Error),
}
