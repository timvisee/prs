use std::io::{stdin, stdout, Write};
use std::process::{Command, Stdio};
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
use prs_lib::{util::env, Plaintext, Secret, Store};
use substring::Substring;
use thiserror::Error;

use crate::cmd::matcher::MainMatcher;
use crate::util::{
    error::{self, ErrorHintsBuilder},
    secret,
};

/// Environment variable to set custom viewer/pager.
const ENV_VAR_PAGER: &str = "PRS_PAGER";

/// Scroll speed when using the mouse wheel.
const SCROLL_SPEED: i16 = 3;

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
    // Use custom viewer when prs pager is configured
    if env::has_non_empty_env(ENV_VAR_PAGER) {
        return pager(plaintext, timeout, matcher_main);
    }

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

    // Timeout, scroll position and text size
    let timeout_at = timeout.map(|t| Instant::now() + t);
    let mut scroll_pos: (u16, u16) = (0, 0);
    let text_size = {
        let text = plaintext.unsecure_to_str().map_err(Err::Utf8)?;
        (
            text.lines().map(|l| l.len()).max().unwrap_or(0),
            text.lines().count(),
        )
    };

    // Viewer drawing loop
    'window: loop {
        // Grab terminal size
        let tty_size = terminal::size().map_err(Err::Size)?;

        // Paint window border
        paint_border(tty_size, &title, timeout_at, matcher_main).map_err(Err::Render)?;

        loop {
            // Painte plaintext
            paint_content(&plaintext, tty_size, scroll_pos, matcher_main).map_err(Err::Render)?;

            // Get actions from input, stop on quit or timeout
            let action = match timeout_at {
                Some(timeout_at) => wait_action_timeout(timeout_at - Instant::now()),
                None => wait_action(),
            };
            match action {
                // Quit or timeout reached
                Some(Action::Quit) | None => break 'window,
                Some(Action::Redraw) => {
                    continue 'window;
                }
                Some(Action::ScrollY(amount)) => {
                    let scroll_max = (text_size.1 as i16 - tty_size.1 as i16
                        + if !matcher_main.quiet() { 2 } else { 0 })
                    .max(0);
                    scroll_pos.1 = (scroll_pos.1 as i16 + amount).clamp(0, scroll_max) as u16;
                }
                Some(Action::ScrollX(amount)) => {
                    let scroll_max = (text_size.0 as i16 - tty_size.0 as i16).max(0);
                    scroll_pos.0 = (scroll_pos.0 as i16 + amount).clamp(0, scroll_max) as u16;
                }
            }
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

/// Paint window borders.
///
/// Doesn't paint anything in quiet mode.
fn paint_border(
    tty_size: (u16, u16),
    title: &str,
    timeout_at: Option<Instant>,
    matcher_main: &MainMatcher,
) -> Result<()> {
    // Don't paint window if quiet
    if matcher_main.quiet() {
        return Ok(());
    }

    // Header
    queue!(stdout(), cursor::MoveTo(0, 0)).map_err(Err::RawTerminal)?;
    print!("{}", banner_text(title, tty_size.0).reverse());

    // Footer
    queue!(stdout(), cursor::MoveTo(0, tty_size.1 - 1)).map_err(Err::RawTerminal)?;
    print!(
        "{}",
        banner_text(
            if let Some(timeout_at) = timeout_at {
                format!(
                    "Press Q to close. Closing in {} seconds...",
                    (timeout_at - Instant::now()).as_secs()
                )
            } else {
                "Press Q to close".to_string()
            },
            tty_size.0
        )
        .reverse()
    );

    stdout().flush().map_err(Err::RawTerminal)?;

    Ok(())
}

/// Paint window contents.
fn paint_content(
    plaintext: &Plaintext,
    tty_size: (u16, u16),
    scroll_pos: (u16, u16),
    matcher_main: &MainMatcher,
) -> Result<()> {
    // Determine viewport size
    let (vw, vh, vy) = if !matcher_main.quiet() {
        (tty_size.0, tty_size.1 - 2, 1)
    } else {
        (tty_size.0, tty_size.1, 0)
    };

    // Get line count and lines iterator
    let (line_count, mut line_iter) = {
        let content = plaintext.unsecure_to_str().map_err(Err::Utf8)?;
        (
            content.lines().count(),
            content.lines().skip(scroll_pos.1 as usize),
        )
    };

    // Paint each line
    for (y, line) in (vy..=vh).map(|y| (y, line_iter.next())) {
        // Set cursor, clear line
        queue!(
            stdout(),
            cursor::MoveTo(0, y),
            terminal::Clear(terminal::ClearType::CurrentLine),
        )
        .map_err(Err::RawTerminal)?;

        // Render tilde if there is no line
        if line.is_none() {
            print!("{}", "~".dark_grey());
            continue;
        }

        // Top scroll marker if there's hidden content
        let first = y == vy;
        if first && scroll_pos.1 > 0 {
            let marker = "^".repeat(vw as usize);
            print!("{}", marker.dark_grey());
            continue;
        }

        // Bottom scroll marker if there's hidden content
        let last = y == vh;
        if last && line_count.saturating_sub(scroll_pos.1 as usize) > vh as usize {
            let marker = "v".repeat(vw as usize);
            print!("{}", marker.dark_grey());
            break;
        }

        let line = line.unwrap();
        let len = line.chars().count();

        let mark_before = scroll_pos.0 > 0;
        let mark_after = len.saturating_sub(scroll_pos.0 as usize) > vw as usize;

        let mut start = scroll_pos.0 as usize;
        let mut end = scroll_pos.0 as usize + vw as usize;

        if mark_before {
            start += 1;
        }
        if mark_after {
            end -= 1;
        }

        if mark_before {
            print!("{}", "<".dark_grey(),);
        }
        print!("{}", line.substring(start, end));
        if mark_after {
            print!("{}", ">".dark_grey(),);
        }
    }

    stdout().flush().map_err(Err::RawTerminal)?;

    Ok(())
}

/// Possible actions.
enum Action {
    /// Quit viewer.
    Quit,

    /// Redraw viewer.
    Redraw,

    /// Scroll horizontal action.
    ScrollX(i16),

    /// Scroll vertical action.
    ScrollY(i16),
}

/// Wait for an action based on a terminal event indefinately.
fn wait_action() -> Option<Action> {
    match event::read() {
        // Quit with Q, Esc or <c-C>
        Ok(event::Event::Key(event::KeyEvent {
            code: event::KeyCode::Char('q') | event::KeyCode::Esc,
            ..
        })) => Some(Action::Quit),
        Ok(event::Event::Key(event::KeyEvent {
            code: event::KeyCode::Char('c'),
            modifiers,
            ..
        })) if modifiers.contains(event::KeyModifiers::CONTROL) => Some(Action::Quit),

        // Scrolling
        Ok(Event::Mouse(event::MouseEvent {
            kind: event::MouseEventKind::ScrollUp,
            ..
        })) => Some(Action::ScrollY(-SCROLL_SPEED)),
        Ok(Event::Mouse(event::MouseEvent {
            kind: event::MouseEventKind::ScrollDown,
            ..
        })) => Some(Action::ScrollY(SCROLL_SPEED)),
        Ok(event::Event::Key(event::KeyEvent {
            code: event::KeyCode::Up | event::KeyCode::Char('k'),
            ..
        })) => Some(Action::ScrollY(-1)),
        Ok(event::Event::Key(event::KeyEvent {
            code: event::KeyCode::Down | event::KeyCode::Char('j'),
            ..
        })) => Some(Action::ScrollY(1)),
        Ok(event::Event::Key(event::KeyEvent {
            code: event::KeyCode::Left | event::KeyCode::Char('h'),
            ..
        })) => Some(Action::ScrollX(-1)),
        Ok(event::Event::Key(event::KeyEvent {
            code: event::KeyCode::Right | event::KeyCode::Char('l'),
            ..
        })) => Some(Action::ScrollX(1)),

        // Resize
        Ok(Event::Resize(_, _) | Event::FocusGained) => Some(Action::Redraw),

        // Ignore other input
        Ok(_) | Err(_) => None,
    }
}

/// Block until a user presses an action key, with timeout.
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

/// Create a banner spanning the whole width.
fn banner_text<S: AsRef<str>>(text: S, width: u16) -> String {
    let text = text.as_ref().trim();

    // Truncate if text is too long
    if text.len() >= width as usize {
        return text.substring(0, width as usize).into();
    }

    let empty = width as usize - text.len();
    let start = empty / 2;
    let end = empty - start;
    let start = " ".repeat(start);
    let end = " ".repeat(end);

    format!("{start}{text}{end}")
}

/// Use custom viewer/pager from `PRS_PAGER`.
fn pager(
    plaintext: Plaintext,
    timeout: Option<Duration>,
    matcher_main: &MainMatcher,
) -> Result<()> {
    // Parse pager arguments, build command
    let args = shlex::split(&std::env::var(ENV_VAR_PAGER).map_err(Err::PagerEnvUtf8)?).unwrap();
    let mut pager = Command::new(&args[0])
        .args(&args[1..])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(Err::PagerSpawn)?;
    pager
        .stdin
        .as_mut()
        .unwrap()
        .write_all(plaintext.unsecure_ref())
        .map_err(Err::PagerPipe)?;

    // Wait for pager to quit
    let status = pager.wait().map_err(Err::PagerSpawn)?;
    if !status.success() {
        return Err(Err::PagerStatus(status).into());
    }

    // Warn if timeout is configured
    if timeout.is_some() && !matcher_main.quiet() {
        error::print_warning("timeout is not supported with custom pager (env: PRS_PAGER)");
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum Err {
    #[error("failed to parse secret contents as UTF-8, required when using viewer")]
    Utf8(#[source] std::str::Utf8Error),

    #[error("failed to manage raw terminal")]
    RawTerminal(#[source] std::io::Error),

    #[error("failed to determine terminal size")]
    Size(#[source] std::io::Error),

    #[error("failed to render secret viewer")]
    Render(#[source] anyhow::Error),

    #[error("failed to parse PRS_PAGER env as UTF-8")]
    PagerEnvUtf8(#[source] std::env::VarError),

    #[error("failed to invoke pager from PRS_PAGER")]
    PagerSpawn(#[source] std::io::Error),

    #[error("failed to pipe secret contents to pager")]
    PagerPipe(#[source] std::io::Error),

    #[error("pager exited with non-zero status code: {0}")]
    PagerStatus(std::process::ExitStatus),
}
