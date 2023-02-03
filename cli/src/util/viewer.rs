use std::io::{stdin, stdout};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event},
    execute, queue, style,
    style::Stylize,
    terminal,
    tty::IsTty,
};
use prs_lib::{Plaintext, Secret, Store};
use thiserror::Error;

use crate::cmd::matcher::MainMatcher;
use crate::util::{
    error::{self, ErrorHintsBuilder},
    secret,
};

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

    // Require to be in a TTY
    if !stdin().is_tty() {
        error::quit_error_msg(
            "secure viewer can only be used in TTY",
            ErrorHintsBuilder::default().verbose(false).build().unwrap(),
        );
    }

    // Get secret name and title
    let name = secret::display_name(query, secret, store, false)
        .unwrap_or_else(|| secret.name.to_string());
    let title = format!("{}: {}", crate::NAME, name);

    // Enter alternative screen now, enable raw mode, queue paint actions
    execute!(stdout(), terminal::EnterAlternateScreen).map_err(Err::RawTerminal)?;
    terminal::enable_raw_mode().map_err(Err::RawTerminal)?;
    queue!(
        stdout(),
        style::ResetColor,
        cursor::Hide,
        terminal::SetTitle(&title),
    )
    .map_err(Err::RawTerminal)?;

    let timeout_at = timeout.map(|t| Instant::now() + t);

    // Viewer drawing loop
    loop {
        paint(&plaintext, &title, timeout_at, matcher_main)?;

        // Get actions from input, stop on quit or timeout
        let action = match timeout_at {
            Some(timeout_at) => wait_action_timeout(timeout_at - Instant::now()),
            None => wait_action(),
        };
        match action {
            // Quit or timeout reached
            Some(Action::Quit) | None => break,
            Some(_) => {}
        }
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

/// Repaint the viewer in a raw terminal.
fn paint(
    plaintext: &Plaintext,
    title: &str,
    timeout_at: Option<Instant>,
    matcher_main: &MainMatcher,
) -> Result<()> {
    // Clear screen and reset cursor
    queue!(
        stdout(),
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0),
    )
    .map_err(Err::RawTerminal)?;

    // Grab terminal size
    let size = terminal::size().map_err(Err::Size)?;

    // Header
    if !matcher_main.quiet() {
        print!("{}\r\n", banner_text(title, size.0).reverse());
    }

    // Update plaintext contents, carriage return is required in raw mode, then print
    let plaintext = plaintext
        .unsecure_to_str()
        .unwrap()
        .replace('\n', "\r\n")
        .into();
    secret::print(plaintext).map_err(Err::Print)?;

    // Footer
    if !matcher_main.quiet() {
        print!(
            "\r\n{}\r\n",
            banner_text(
                if let Some(timeout_at) = timeout_at {
                    format!(
                        "Press Q to close. Closing in {} seconds...",
                        (timeout_at - Instant::now()).as_secs()
                    )
                } else {
                    "Press Q to close".to_string()
                },
                size.0
            )
            .reverse()
        );
    }

    Ok(())
}

/// Possible actions.
enum Action {
    /// Quit viewer.
    Quit,

    /// Redraw viewer.
    Redraw,
}

/// Wait for an action based on a terminal event indefinately.
fn wait_action() -> Option<Action> {
    match event::read() {
        Ok(event) if is_quit_key(event) => Some(Action::Quit),
        Ok(Event::Resize(_, _) | Event::FocusGained) => Some(Action::Redraw),
        Ok(_) | Err(_) => None,
    }
}

/// Block until a user presses a quit key, with timeout.
///
/// See `is_quit_key` for a list of keys.
fn wait_action_timeout(timeout: Duration) -> Option<Action> {
    let until = Instant::now() + timeout;
    loop {
        // Return if timeout is reached
        let now = Instant::now();
        if until <= now {
            return None;
        }

        // Poll, stop if timeout is reached or on error
        if matches!(event::poll(until - now), Ok(false) | Err(_)) {
            return None;
        }

        if let Some(input) = wait_action() {
            return Some(input);
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

/// Create a banner spanning the whole width.
fn banner_text<S: AsRef<str>>(text: S, width: u16) -> String {
    let text = text.as_ref().trim();

    // Truncate if text is too long
    if text.len() >= width as usize {
        let mut text = text.to_string();
        text.truncate(width as usize);
        return text;
    }

    // TODO: this is very inefficient, use better way
    let left = width as usize - text.len();
    let before = left / 2;
    let after = left - before;
    let before = std::iter::repeat(" ")
        .take(before)
        .collect::<Vec<_>>()
        .join("");
    let after = std::iter::repeat(" ")
        .take(after)
        .collect::<Vec<_>>()
        .join("");

    format!("{before}{text}{after}")
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to manage raw terminal")]
    RawTerminal(#[source] std::io::Error),

    #[error("failed to determine terminal size")]
    Size(#[source] std::io::Error),

    #[error("failed to print secret to viewer")]
    Print(#[source] std::io::Error),
}
