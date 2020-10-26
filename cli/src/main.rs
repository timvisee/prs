#[macro_use]
extern crate clap;
#[macro_use]
extern crate derive_builder;

mod action;
mod cmd;
mod util;

use anyhow::Result;

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

    if handler.edit().is_some() {
        return action::edit::Edit::new(handler.matches()).invoke();
    }

    if handler.generate().is_some() {
        return action::generate::Generate::new(handler.matches()).invoke();
    }

    if handler.r#move().is_some() {
        return action::r#move::Move::new(handler.matches()).invoke();
    }

    if handler.list().is_some() {
        return action::list::List::new(handler.matches()).invoke();
    }

    if handler.new().is_some() {
        return action::new::New::new(handler.matches()).invoke();
    }

    if handler.show().is_some() {
        return action::show::Show::new(handler.matches()).invoke();
    }

    Ok(())
}
