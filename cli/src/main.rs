#[macro_use]
extern crate clap;

mod action;
mod cmd;

use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

use skim::{
    prelude::{SkimItemReceiver, SkimItemSender, SkimOptionsBuilder},
    AnsiString, Skim, SkimItem,
};

use passr::store::{Entry, Store};

use crate::cmd::Handler;

const STORE_DEFAULT_ROOT: &str = "~/.password-store";

fn main() {
    // Parse CLI arguments
    let cmd_handler = Handler::parse();

    // Invoke the proper action
    if let Err(err) = invoke_action(&cmd_handler) {
        panic!("action failure: {:?}", err);
        // TODO: quit_error(err, ErrorHints::default());
    };
}

/// Invoke the proper action based on the CLI input.
///
/// If no proper action is selected, the program will quit with an error
/// message.
fn invoke_action(handler: &Handler) -> Result<(), ()> {
    // Match the debug command
    if handler.show().is_some() {
        return action::show::Show::new(handler.matches())
            .invoke()
            .map_err(|err| err.into());
    }

    Ok(())
}

/// Show an interactive selection view for the given list of `items`.
/// The selected item is returned.  If no item is selected, `None` is returned instead.
fn select(items: SkimItemReceiver, prompt: &str) -> Option<String> {
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

/// Wrapped store entry item for skim.
pub struct SkimEntry(Entry);

impl From<Entry> for SkimEntry {
    fn from(entry: Entry) -> Self {
        Self(entry)
    }
}

impl SkimItem for SkimEntry {
    fn display(&self) -> Cow<AnsiString> {
        let s: AnsiString = self.0.name().clone().into();
        Cow::Owned(s)
    }

    fn text(&self) -> Cow<str> {
        self.0.name().into()
    }

    fn output(&self) -> Cow<str> {
        self.0.path().to_string_lossy()
    }
}

/// Select entry.
fn select_entry(entries: &[Entry]) -> Option<&Entry> {
    // Let user select entry
    let items = skim_entry_items(entries);
    let selected = select(items, "Select entry")?;

    // Pick selected item from entries list
    let path: PathBuf = selected.into();
    Some(entries.iter().find(|e| e.path() == path).unwrap())
}

/// Generate skim `SkimEntry` items from given entries.
fn skim_entry_items(entries: &[Entry]) -> SkimItemReceiver {
    let entries: Vec<SkimEntry> = entries.iter().cloned().map(|e| e.into()).collect();

    let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) =
        skim::prelude::bounded(entries.len());

    entries.into_iter().for_each(|g| {
        let _ = tx_item.send(Arc::new(g));
    });

    rx_item
}
