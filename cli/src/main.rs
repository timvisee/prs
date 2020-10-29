#[macro_use]
extern crate clap;
#[macro_use]
extern crate derive_builder;

mod action;
mod cmd;
mod util;

use std::process;

use anyhow::Result;
use prs_lib::store::Store;

use crate::{
    cmd::matcher::{MainMatcher, Matcher},
    cmd::Handler,
    util::{
        error::{quit_error, ErrorHints},
        style,
    },
};

/// Default password store directory.
const STORE_DEFAULT_ROOT: &str = "~/.password-store";

/// Clipboard timeout in seconds.
const CLIPBOARD_TIMEOUT_STR: &str = "20";

fn main() {
    // Parse CLI arguments
    let cmd_handler = Handler::parse();

    // Invoke the proper action
    if let Err(err) = invoke_action(&cmd_handler) {
        quit_error(err, ErrorHints::default());
    };
}

/// Invoke the proper action based on the CLI input.
///
/// If no proper action is selected, the program will quit with an error
/// message.
fn invoke_action(handler: &Handler) -> Result<()> {
    if handler.add().is_some() {
        return action::add::Add::add(handler.matches()).invoke();
    }

    if handler.clone().is_some() {
        return action::clone::Clone::new(handler.matches()).invoke();
    }

    if handler.copy().is_some() {
        return action::copy::Copy::new(handler.matches()).invoke();
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

    if handler.git().is_some() {
        return action::git::Git::new(handler.matches()).invoke();
    }

    if handler.housekeeping().is_some() {
        return action::housekeeping::Housekeeping::new(handler.matches()).invoke();
    }

    if handler.r#move().is_some() {
        return action::r#move::Move::new(handler.matches()).invoke();
    }

    if handler.init().is_some() {
        return action::init::Init::new(handler.matches()).invoke();
    }

    if handler.list().is_some() {
        return action::list::List::new(handler.matches()).invoke();
    }

    if handler.recipients().is_some() {
        return action::recipients::Recipients::new(handler.matches()).invoke();
    }

    if handler.remove().is_some() {
        return action::remove::Remove::new(handler.matches()).invoke();
    }

    if handler.show().is_some() {
        return action::show::Show::new(handler.matches()).invoke();
    }

    if handler.sync().is_some() {
        return action::sync::Sync::new(handler.matches()).invoke();
    }

    // Get the main matcher
    let matcher_main = MainMatcher::with(handler.matches()).unwrap();
    if !matcher_main.quiet() {
        print_main_info();
    }

    Ok(())
}

/// Print the main info, shown when no subcommands were supplied.
pub fn print_main_info() -> ! {
    // Get the name of the used executable
    let bin = util::bin_name();

    // Attempt to load default store
    let store = Store::open(crate::STORE_DEFAULT_ROOT).ok();
    let has_sync = store.as_ref().map(|s| s.sync().is_init()).unwrap_or(false);

    // Print the main info
    println!("{} {}", crate_name!(), crate_version!());
    println!("Usage: {} [FLAGS] <SUBCOMMAND> ...", bin);
    println!(crate_description!());
    println!();

    if store.is_none() {
        println!("Initialize a new password store or clone an existing one:");
        println!("    {}", style::highlight(&format!("{} init", bin)));
        println!(
            "    {}",
            style::highlight(&format!("{} clone <GIT_URL>", bin))
        );
        println!();
    } else {
        let store = store.unwrap();

        // Hint user to add ourselves as recipient if it doesn't have recipient we own
        let we_own_any_recipient = store
            .recipients()
            .and_then(|recip| prs_lib::contains_own_secret_key(&recip))
            .unwrap_or(false);
        if !we_own_any_recipient {
            let system_has_secret = action::clone::has_secret_key_in_keychain().unwrap_or(true);
            if system_has_secret {
                println!("Add your own key as recipient or generate a new one:");
            } else {
                println!("Generate and add a new recipient key for yourself:");
            }
            if system_has_secret {
                println!(
                    "    {}",
                    style::highlight(&format!("{} recipients add --secret", bin))
                );
            }
            println!(
                "    {}",
                style::highlight(&format!("{} recipients generate", bin))
            );
            println!();
        }

        // Hint show/copy commands if user has secret
        let has_secret = store.secret_iter().next().is_some();
        if has_secret {
            println!("Show or copy a secret:");
            println!("    {}", style::highlight(&format!("{} show [NAME]", bin)));
            println!("    {}", style::highlight(&format!("{} copy [NAME]", bin)));
            println!();
        }

        // Hint add/edit/remove commands if store has recipient we own
        if we_own_any_recipient {
            println!("Generate, add, edit or remove secrets:");
            println!(
                "    {}",
                style::highlight(&format!("{} generate <NAME>", bin))
            );
            println!("    {}", style::highlight(&format!("{} add <NAME>", bin)));
            println!("    {}", style::highlight(&format!("{} edit [NAME]", bin)));
            println!(
                "    {}",
                style::highlight(&format!("{} remove [NAME]", bin))
            );
            println!();
        }

        // Hint about sync
        if has_sync {
            println!("Sync your password store:");
            println!("    {}", style::highlight(&format!("{} sync", bin)));
            println!();
        } else {
            println!("Enable sync for your password store:");
            println!("    {}", style::highlight(&format!("{} sync init", bin)));
            println!();
        }
    }

    println!("Show all subcommands, features and other help:");
    println!(
        "    {}",
        style::highlight(&format!("{} help [SUBCOMMAND]", bin))
    );

    process::exit(1)
}
