use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

use prs_lib::{
    store::{FindSecret, Secret, Store},
    Key,
};
use skim::{
    prelude::{SkimItemReceiver, SkimItemSender, SkimOptionsBuilder},
    AnsiString, Skim, SkimItem,
};

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
