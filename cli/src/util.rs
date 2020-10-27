// TODO: remove this when somewhat feature complete
#![allow(unused)]

use std::borrow::{Borrow, Cow};
use std::env::{self, current_exe, var_os};
use std::ffi::OsStr;
use std::fmt::{Debug, Display};
use std::io::{self, stderr, stdin, Error as IoError, Read, Write};
use std::iter;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::{exit, ExitStatus};
use std::sync::Arc;

use anyhow::{anyhow, Error, Result};
use colored::{ColoredString, Colorize};
use skim::{
    prelude::{SkimItemReceiver, SkimItemSender, SkimOptionsBuilder},
    AnsiString, Skim, SkimItem,
};

use prs_lib::{
    store::{FindSecret, Secret, Store},
    types::Plaintext,
    Key,
};

use crate::cmd::matcher::MainMatcher;

/// Print a success message.
pub fn print_success(msg: &str) {
    eprintln!("{}", msg.green());
}

/// Print the given error in a proper format for the user,
/// with it's causes.
pub fn print_error(err: Error) {
    // Report each printable error, count them
    let count = err
        .chain()
        .map(|err| format!("{}", err))
        .filter(|err| !err.is_empty())
        .enumerate()
        .map(|(i, err)| {
            if i == 0 {
                eprintln!("{} {}", highlight_error("error:"), err);
            } else {
                eprintln!("{} {}", highlight_error("caused by:"), err);
            }
        })
        .count();

    // Fall back to a basic message
    if count == 0 {
        eprintln!(
            "{} {}",
            highlight_error("error:"),
            "an undefined error occurred"
        );
    }
}

/// Print the given error message in a proper format for the user,
/// with it's causes.
pub fn print_error_msg<S>(err: S)
where
    S: AsRef<str> + Display + Debug + Sync + Send + 'static,
{
    print_error(anyhow!(err));
}

/// Print a warning.
pub fn print_warning<S>(err: S)
where
    S: AsRef<str> + Display + Debug + Sync + Send + 'static,
{
    eprintln!("{} {}", highlight_warning("warning:"), err);
}

/// Quit the application regularly.
pub fn quit() -> ! {
    exit(0);
}

/// Quit the application with an error code,
/// and print the given error.
pub fn quit_error(err: Error, hints: impl Borrow<ErrorHints>) -> ! {
    // Print the error
    print_error(err);

    // Print error hints
    hints.borrow().print();

    // Quit
    exit(1);
}

/// Quit the application with an error code,
/// and print the given error message.
pub fn quit_error_msg<S>(err: S, hints: impl Borrow<ErrorHints>) -> !
where
    S: AsRef<str> + Display + Debug + Sync + Send + 'static,
{
    quit_error(anyhow!(err), hints);
}

/// The error hint configuration.
#[derive(Clone, Builder)]
#[builder(default)]
pub struct ErrorHints {
    /// A list of info messages to print along with the error.
    info: Vec<String>,

    /// Show about the sync action.
    sync: bool,

    /// Show about the force flag.
    force: bool,

    /// Show about the verbose flag.
    verbose: bool,

    /// Show about the help flag.
    help: bool,
}

impl ErrorHints {
    /// Check whether any hint should be printed.
    pub fn any(&self) -> bool {
        self.sync || self.force || self.verbose || self.help
    }

    /// Print the error hints.
    pub fn print(&self) {
        // Print info messages
        for msg in &self.info {
            eprintln!("{} {}", highlight_info("info:"), msg);
        }

        // Stop if nothing should be printed
        if !self.any() {
            return;
        }

        eprint!("\n");

        // Print hints
        if self.sync {
            eprintln!(
                "To sync your your password store use '{}'",
                // TODO: use current program name here
                highlight(&format!("{} sync", crate_name!()))
            );
        }
        if self.force {
            eprintln!("Use '{}' to force", highlight("--force"));
        }
        if self.verbose {
            eprintln!("For detailed errors try '{}'", highlight("--verbose"));
        }
        if self.help {
            eprintln!("For more information try '{}'", highlight("--help"));
        }

        // Flush
        let _ = stderr().flush();
    }
}

impl Default for ErrorHints {
    fn default() -> Self {
        ErrorHints {
            info: Vec::new(),
            sync: false,
            force: false,
            verbose: true,
            help: true,
        }
    }
}

impl ErrorHintsBuilder {
    /// Add a single info entry.
    pub fn add_info(mut self, info: String) -> Self {
        // Initialize the info list
        if self.info.is_none() {
            self.info = Some(Vec::new());
        }

        // Add the item to the info list
        if let Some(ref mut list) = self.info {
            list.push(info);
        }

        self
    }
}

/// Highlight the given text with a color.
pub fn highlight(msg: &str) -> ColoredString {
    msg.yellow()
}

/// Highlight the given text with an error color.
pub fn highlight_error(msg: &str) -> ColoredString {
    msg.red().bold()
}

/// Highlight the given text with an warning color.
pub fn highlight_warning(msg: &str) -> ColoredString {
    highlight(msg).bold()
}

/// Highlight the given text with an info color
pub fn highlight_info(msg: &str) -> ColoredString {
    msg.cyan()
}

/// Prompt the user to enter some value.
/// The prompt that is shown should be passed to `msg`,
/// excluding the `:` suffix.
pub fn prompt(msg: &str, main_matcher: &MainMatcher) -> String {
    // Quit with an error if we may not interact
    if main_matcher.no_interact() {
        quit_error_msg(
            format!(
                "could not prompt for '{}' in no-interact mode, maybe specify it",
                msg,
            ),
            ErrorHints::default(),
        );
    }

    // Show the prompt
    eprint!("{}: ", msg);
    let _ = stderr().flush();

    // Get the input
    let mut input = String::new();
    if let Err(err) = stdin()
        .read_line(&mut input)
        .map_err(|err| -> Error { err.into() })
    {
        quit_error(
            err.context("failed to read input from prompt"),
            ErrorHints::default(),
        );
    }

    // Trim and return
    input.trim().to_owned()
}

/// Prompt the user for a question, allowing a yes or now answer.
/// True is returned if yes was answered, false if no.
///
/// A default may be given, which is chosen if no-interact mode is
/// enabled, or if enter was pressed by the user without entering anything.
pub fn prompt_yes(msg: &str, def: Option<bool>, main_matcher: &MainMatcher) -> bool {
    // Define the available options string
    let options = format!(
        "[{}/{}]",
        match def {
            Some(def) if def => "Y",
            _ => "y",
        },
        match def {
            Some(def) if !def => "N",
            _ => "n",
        }
    );

    // Assume yes
    if main_matcher.assume_yes() {
        eprintln!("{} {}: yes", msg, options);
        return true;
    }

    // Autoselect if in no-interact mode
    if main_matcher.no_interact() {
        if let Some(def) = def {
            eprintln!("{} {}: {}", msg, options, if def { "yes" } else { "no" });
            return def;
        } else {
            quit_error_msg(
                format!(
                    "could not prompt question '{}' in no-interact mode, maybe specify it",
                    msg,
                ),
                ErrorHints::default(),
            );
        }
    }

    // Get the user input
    let answer = prompt(&format!("{} {}", msg, options), main_matcher);

    // Assume the default if the answer is empty
    if answer.is_empty() && def.is_some() {
        return def.unwrap();
    }

    // Derive a boolean and return
    match derive_bool(&answer) {
        Some(answer) => answer,
        None => prompt_yes(msg, def, main_matcher),
    }
}

/// Try to derive true or false (yes or no) from the given input.
/// None is returned if no boolean could be derived accurately.
fn derive_bool(input: &str) -> Option<bool> {
    // Process the input
    let input = input.trim().to_lowercase();

    // Handle short or incomplete answers
    match input.as_str() {
        "y" | "ye" | "t" | "1" => return Some(true),
        "n" | "f" | "0" => return Some(false),
        _ => {}
    }

    // Handle complete answers with any suffix
    if input.starts_with("yes") || input.starts_with("true") {
        return Some(true);
    }
    if input.starts_with("no") || input.starts_with("false") {
        return Some(false);
    }

    // The answer could not be determined, return none
    None
}

/// Edit given plaintext in default editor.
///
/// Only returns `Plaintext` if changed.
pub fn edit(plaintext: &Plaintext) -> Result<Option<Plaintext>, std::io::Error> {
    edit::edit_bytes(&plaintext.0).map(|data| {
        Some(data)
            .filter(|data| &plaintext.0 != data)
            .map(Plaintext)
    })
}

/// Find and select a secret in the given store.
///
/// If no exact secret is found, the user will be able to choose.
///
/// `None` is returned if no secret was found or selected.
pub fn select_secret(store: &Store, query: Option<String>) -> Option<Secret> {
    // TODO: do not use interactive selection with --no-interact mode
    match store.find(query) {
        FindSecret::Exact(secret) => Some(secret),
        FindSecret::Many(secrets) => skim_select_secret(&secrets).cloned(),
    }
}

/// Show an interactive selection view for the given list of `items`.
/// The selected item is returned.  If no item is selected, `None` is returned instead.
fn skim_select(items: SkimItemReceiver, prompt: &str) -> Option<String> {
    let prompt = format!("{}: ", prompt);
    let options = SkimOptionsBuilder::default()
        .prompt(Some(&prompt))
        .height(Some("50%"))
        .multi(false)
        .build()
        .unwrap();

    let selected = Skim::run_with(&options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_else(|| Vec::new());

    // Get the first selected, and return
    selected.iter().next().map(|i| i.output().to_string())
}

/// Wrapped store secret item for skim.
pub struct SkimSecret(Secret);

impl From<Secret> for SkimSecret {
    fn from(secret: Secret) -> Self {
        Self(secret)
    }
}

impl SkimItem for SkimSecret {
    fn display(&self) -> Cow<AnsiString> {
        let s: AnsiString = self.0.name.clone().into();
        Cow::Owned(s)
    }

    fn text(&self) -> Cow<str> {
        (&self.0.name).into()
    }

    fn output(&self) -> Cow<str> {
        self.0.path.to_string_lossy()
    }
}

/// Select secret.
fn skim_select_secret(secrets: &[Secret]) -> Option<&Secret> {
    // Return if theres just one to choose
    if secrets.len() == 1 {
        return secrets.get(0);
    }

    // Let user select secret
    let items = skim_secret_items(secrets);
    let selected = skim_select(items, "Select secret")?;

    // Pick selected item from secrets list
    let path: PathBuf = selected.into();
    Some(secrets.iter().find(|e| e.path == path).unwrap())
}

/// Select key.
pub fn skim_select_key(keys: &[Key]) -> Option<&Key> {
    // Let user select secret
    let items = skim_key_items(keys);
    let selected = skim_select(items, "Select key")?;

    // Pick selected item from keys list
    Some(
        keys.iter()
            .find(|e| e.fingerprint(false) == selected)
            .unwrap(),
    )
}

/// Generate skim `SkimSecret` items from given secrets.
fn skim_secret_items(secrets: &[Secret]) -> SkimItemReceiver {
    skim_items(
        secrets
            .iter()
            .cloned()
            .map(|e| e.into())
            .collect::<Vec<SkimSecret>>(),
    )
}

/// Generate skim `SkimSecret` items from given secrets.
fn skim_key_items(keys: &[Key]) -> SkimItemReceiver {
    skim_items(
        keys.iter()
            .cloned()
            .map(|e| e.into())
            .collect::<Vec<SkimKey>>(),
    )
}

/// Create `SkimItemReceiver` from given array.
fn skim_items<I: SkimItem>(items: Vec<I>) -> SkimItemReceiver {
    let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) =
        skim::prelude::bounded(items.len());

    items.into_iter().for_each(|g| {
        let _ = tx_item.send(Arc::new(g));
    });

    rx_item
}

/// Wrapped store key item for skim.
pub struct SkimKey(Key);

impl From<Key> for SkimKey {
    fn from(key: Key) -> Self {
        Self(key)
    }
}

impl SkimItem for SkimKey {
    fn display(&self) -> Cow<AnsiString> {
        let s: AnsiString = format!("{}", self.0).into();
        Cow::Owned(s)
    }

    fn text(&self) -> Cow<str> {
        format!("{}", self.0).into()
    }

    fn output(&self) -> Cow<str> {
        self.0.fingerprint(false).into()
    }
}

/// Read file from stdin.
fn stdin_read_file(prompt: bool) -> Vec<u8> {
    if prompt {
        eprintln!("Enter input. Use [CTRL+D] to stop:");
    }

    let mut data = vec![];
    io::stdin().lock().read_to_end(&mut data);
    data
}

/// Read plaintext from stdin.
pub fn stdin_read_plaintext(prompt: bool) -> Plaintext {
    Plaintext(stdin_read_file(prompt))
}

/// Invoke a command.
///
/// Quit on error.
pub fn invoke_cmd(cmd: String, dir: Option<&Path>, verbose: bool) -> Result<(), std::io::Error> {
    if verbose {
        eprintln!("Invoking: {}\n", cmd);
    }

    // Invoke command
    // TODO: make this compatible with Windows
    let mut process = Command::new("sh");
    process.arg("-c").arg(&cmd);
    if let Some(dir) = dir {
        process.current_dir(dir);
    }
    let status = process.status()?;

    // Report status errors
    if !status.success() {
        eprintln!();
        quit_error_msg(
            format!(
                "{} exited with status code {}",
                cmd.trim_start().split(" ").next().unwrap_or("command"),
                status.code().unwrap_or(-1)
            ),
            ErrorHints::default(),
        );
    }

    Ok(())
}
