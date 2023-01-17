use std::io::stdout;
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::ArgMatches;
use crossterm::{
    cursor,
    event::{self, Event},
    execute, queue, style,
    style::Stylize,
    terminal,
};
use prs_lib::{crypto::prelude::*, Plaintext, Secret, Store};
use thiserror::Error;

use crate::cmd::matcher::{show::ShowMatcher, MainMatcher, Matcher};
#[cfg(feature = "clipboard")]
use crate::util::clipboard;
#[cfg(all(feature = "tomb", target_os = "linux"))]
use crate::util::tomb;
use crate::util::{secret, select};

/// Show secret action.
pub struct Show<'a> {
    cmd_matches: &'a ArgMatches,
}

impl<'a> Show<'a> {
    /// Construct a new show action.
    pub fn new(cmd_matches: &'a ArgMatches) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the show action.
    pub fn invoke(&self) -> Result<()> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_show = ShowMatcher::with(self.cmd_matches).unwrap();

        let store = Store::open(matcher_main.store()).map_err(Err::Store)?;
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        let mut tomb = store.tomb(
            !matcher_main.verbose(),
            matcher_main.verbose(),
            matcher_main.force(),
        );

        // Prepare tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::prepare_tomb(&mut tomb, &matcher_main).map_err(Err::Tomb)?;

        let secret =
            select::store_select_secret(&store, matcher_show.query()).ok_or(Err::NoneSelected)?;

        let mut plaintext = crate::crypto::context(&matcher_main)?
            .decrypt_file(&secret.path)
            .map_err(Err::Read)?;

        // Trim plaintext to first line or property
        if matcher_show.first_line() {
            plaintext = plaintext.first_line()?;
        } else if let Some(property) = matcher_show.property() {
            plaintext = plaintext.property(property).map_err(Err::Property)?;
        }

        // Copy to clipboard
        #[cfg(feature = "clipboard")]
        if matcher_show.copy() {
            clipboard::plaintext_copy(
                plaintext.clone(),
                true,
                !matcher_main.force(),
                !matcher_main.quiet(),
                matcher_show
                    .timeout()
                    .unwrap_or(Ok(crate::CLIPBOARD_TIMEOUT))?,
            )?;
        }

        // Show directly or with timeout
        match matcher_show.timeout() {
            None => {
                secret::print_name(matcher_show.query(), &secret, &store, matcher_main.quiet());
                secret::print(plaintext).map_err(Err::Print)?
            }
            Some(sec) => show_timeout(
                &store,
                &secret,
                plaintext,
                Duration::from_secs(sec?),
                &matcher_main,
                matcher_show.query(),
            )
            .map_err(Err::ShowTimeout)?,
        }

        // Finalize tomb
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        tomb::finalize_tomb(&mut tomb, &matcher_main, false).map_err(Err::Tomb)?;

        Ok(())
    }
}

/// Show plaintext with timeout.
// TODO: error reporting!
pub(crate) fn show_timeout(
    store: &Store,
    secret: &Secret,
    plaintext: Plaintext,
    timeout: Duration,
    matcher_main: &MainMatcher,
    query: Option<String>,
) -> Result<()> {
    // Don't show anything if empty
    if plaintext.is_empty() & !matcher_main.force() {
        if matcher_main.verbose() {
            eprintln!("Secret is empty");
        }
        return Ok(());
    }

    // Get secret name and title
    let name = secret::display_name(query, secret, store, false)
        .unwrap_or_else(|| secret.name.to_string());
    let title = format!("{}: {}", crate::NAME, name);

    // Enter alternative screen now, queue
    execute!(stdout(), terminal::EnterAlternateScreen).map_err(Err::RawTerminal)?;
    queue!(
        stdout(),
        style::ResetColor,
        terminal::Clear(terminal::ClearType::All),
        cursor::Hide,
        cursor::MoveTo(0, 0),
        terminal::SetTitle(&title),
    )
    .map_err(Err::RawTerminal)?;

    // Header
    if !matcher_main.quiet() {
        println!("{}", title.reverse());
    }

    // Secret contents
    secret::print(plaintext).map_err(Err::Print)?;

    // Footer
    if !matcher_main.quiet() {
        println!(
            "\n{}",
            format!(
                "Press Q to close. Closing in {} seconds...",
                timeout.as_secs()
            )
            .reverse()
        );
    }

    // Enable raw input after printing to catch input
    terminal::enable_raw_mode().map_err(Err::RawTerminal)?;

    // Wait for quit key or timeout
    wait_quit_key_timeout(timeout);

    // Clean up alternative screen, switch back to main
    execute!(
        stdout(),
        terminal::Clear(terminal::ClearType::Purge),
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen,
    )
    .map_err(Err::RawTerminal)?;
    terminal::disable_raw_mode().map_err(Err::RawTerminal)?;

    Ok(())
}

/// Block until a user presses a quit key.
///
/// See `is_quit_key` for a list of keys.
fn wait_quit_key() {
    while !event::read().map(is_quit_key).unwrap_or(false) {}
}

/// Block until a user presses a quit key, with timeout.
///
/// See `is_quit_key` for a list of keys.
fn wait_quit_key_timeout(timeout: Duration) {
    let until = Instant::now() + timeout;
    loop {
        // Return if timeout is reached
        let now = Instant::now();
        if until <= now {
            return;
        }

        // Poll, stop if timeout is reached or on error
        if matches!(event::poll(until - now), Ok(false) | Err(_)) {
            return;
        }

        // Stop if quit key is pressed
        if event::read().map(is_quit_key).unwrap_or(false) {
            return;
        }
    }
}

/// Check if the given event is for a quit key.
///
/// - Q
/// - Escape
/// - CTRL+C
fn is_quit_key(event: Event) -> bool {
    match event {
        // Q or Escape key
        event::Event::Key(event::KeyEvent {
            code: event::KeyCode::Char('q') | event::KeyCode::Esc,
            ..
        }) => true,
        // CTRL+C
        event::Event::Key(event::KeyEvent {
            code: event::KeyCode::Char('c'),
            modifiers,
            ..
        }) if modifiers.contains(event::KeyModifiers::CONTROL) => true,
        _ => false,
    }
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to access password store")]
    Store(#[source] anyhow::Error),

    #[cfg(all(feature = "tomb", target_os = "linux"))]
    #[error("failed to prepare password store tomb for usage")]
    Tomb(#[source] anyhow::Error),

    #[error("no secret selected")]
    NoneSelected,

    #[error("failed to read secret")]
    Read(#[source] anyhow::Error),

    #[error("failed to select property from secret")]
    Property(#[source] anyhow::Error),

    #[error("failed to print secret to stdout")]
    Print(#[source] std::io::Error),

    #[error("failed to show secret in viewer with timeout")]
    ShowTimeout(#[source] anyhow::Error),

    #[error("failed to manage raw terminal")]
    RawTerminal(#[source] std::io::Error),
}
