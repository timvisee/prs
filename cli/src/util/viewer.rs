use std::io::stdout;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event},
    execute, queue, style,
    style::Stylize,
    terminal,
};
use prs_lib::{Plaintext, Secret, Store};
use thiserror::Error;

use crate::cmd::matcher::MainMatcher;
use crate::util::secret;

/// Show plaintext in secure viewer with optional timeout.
///
/// This shows plaintext in an alternative terminal buffer, in order to keep its contents as
/// secure as possible. All data is cleared when the viewer is closed.
///
/// The user may quit using a quit key, or the viewer may time out.
pub(crate) fn viewer(
    store: &Store,
    secret: &Secret,
    plaintext: Plaintext,
    timeout: Option<Duration>,
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
            if let Some(timeout) = timeout {
                format!(
                    "Press Q to close. Closing in {} seconds...",
                    timeout.as_secs()
                )
            } else {
                format!("Press Q to close",)
            }
            .reverse()
        );
    }

    // Enable raw input after printing to catch input
    terminal::enable_raw_mode().map_err(Err::RawTerminal)?;

    // Wait for quit key or timeout
    match timeout {
        Some(timeout) => wait_quit_key_timeout(timeout),
        None => wait_quit_key(),
    }

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
    #[error("failed to manage raw terminal")]
    RawTerminal(#[source] std::io::Error),

    #[error("failed to print secret to viewer")]
    Print(#[source] std::io::Error),
}
