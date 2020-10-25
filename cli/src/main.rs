#[macro_use]
extern crate clap;
#[macro_use]
extern crate derive_builder;

mod action;
mod cmd;
mod util;

use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use skim::{
    prelude::{SkimItemReceiver, SkimItemSender, SkimOptionsBuilder},
    AnsiString, Skim, SkimItem,
};

use prs_lib::store::Secret;

use crate::cmd::Handler;

const STORE_DEFAULT_ROOT: &str = "~/.password-store";

fn main() {
    // Parse CLI arguments
    let cmd_handler = Handler::parse();

    // Invoke the proper action
    if let Err(err) = invoke_action(&cmd_handler) {
        util::quit_error(err, util::ErrorHints::default());
    };
}

/// Invoke the proper action based on the CLI input.
///
/// If no proper action is selected, the program will quit with an error
/// message.
fn invoke_action(handler: &Handler) -> Result<()> {
    if handler.copy().is_some() {
        return action::copy::Copy::new(handler.matches()).invoke();
    }

    if handler.delete().is_some() {
        return action::delete::Delete::new(handler.matches()).invoke();
    }

    if handler.duplicate().is_some() {
        return action::duplicate::Duplicate::new(handler.matches()).invoke();
    }

    if handler.r#move().is_some() {
        return action::r#move::Move::new(handler.matches()).invoke();
    }

    if handler.list().is_some() {
        return action::list::List::new(handler.matches()).invoke();
    }

    if handler.show().is_some() {
        return action::show::Show::new(handler.matches()).invoke();
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
fn select_secret(secrets: &[Secret]) -> Option<&Secret> {
    // Return if theres just one to choose
    if secrets.len() == 1 {
        return secrets.get(0);
    }

    // Let user select secret
    let items = skim_secret_items(secrets);
    let selected = select(items, "Select secret")?;

    // Pick selected item from secrets list
    let path: PathBuf = selected.into();
    Some(secrets.iter().find(|e| e.path == path).unwrap())
}

/// Generate skim `SkimSecret` items from given secrets.
fn skim_secret_items(secrets: &[Secret]) -> SkimItemReceiver {
    let items: Vec<SkimSecret> = secrets.iter().cloned().map(|e| e.into()).collect();

    let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) =
        skim::prelude::bounded(items.len());

    items.into_iter().for_each(|g| {
        let _ = tx_item.send(Arc::new(g));
    });

    rx_item
}
