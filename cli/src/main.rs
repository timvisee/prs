use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

use skim::{
    prelude::{SkimItemReceiver, SkimItemSender, SkimOptionsBuilder},
    AnsiString, Skim, SkimItem,
};

use passr::store::{Entry, Store};

const STORE_DEFAULT_ROOT: &str = "~/.password-store";

fn main() {
    // Open password store, get recipients
    let store = Store::open(STORE_DEFAULT_ROOT);
    // let recipients = store.recipients().expect("failed to list recipients");

    // // Test encrypt & decrypt
    // passr::crypto::encrypt_file(&recipients, plaintext, &path).expect("failed to encrypt");
    // let plaintext = passr::crypto::decrypt_file(&path).expect("failed to decrypt");

    // println!("=v=v=v=v=v=v=v=v=v=");
    // std::io::stdout().write_all(&plaintext.0).unwrap();
    // println!("\n=^=^=^=^=^=^=^=^=^=");

    let entries = store.entries();
    let entry = select_entry(&entries);
    dbg!(entry);
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

/// Select game.
// TODO: use ref
// TODO: handle no selection
fn select_entry(entries: &[Entry]) -> &Entry {
    // Find game directories
    // TODO: improve this
    let game_items = skim_entry_items(entries);

    let selected = select(game_items, "Select entry").expect("did not select entry");
    let dir: PathBuf = selected.into();

    entries.iter().find(|e| e.path() == dir).unwrap()
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

/// Generate skim `SkimEntry` from given entries.
fn skim_entry_items(entries: &[Entry]) -> SkimItemReceiver {
    // Transform into skim entries
    // TODO: do not clone
    let entries: Vec<SkimEntry> = entries.into_iter().map(|e| e.clone().into()).collect();

    let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) =
        skim::prelude::bounded(entries.len());

    entries.into_iter().for_each(|g| {
        let _ = tx_item.send(Arc::new(g));
    });

    rx_item
}
