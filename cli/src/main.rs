#[macro_use]
extern crate clap;
#[macro_use]
extern crate derive_builder;
#[allow(unused_imports)]
#[macro_use]
extern crate lazy_static;

mod action;
mod cmd;
mod crypto;
mod util;

use anyhow::Result;
use prs_lib::Store;

use crate::{
    cmd::matcher::{MainMatcher, Matcher},
    cmd::Handler,
    util::{
        error::{quit, quit_error, ErrorHints},
        style,
    },
};

/// Clipboard timeout in seconds.
#[cfg(feature = "clipboard")]
const CLIPBOARD_TIMEOUT: u64 = 20;

fn main() {
    // Do not use colored output on Windows
    #[cfg(windows)]
    colored::control::set_override(false);

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
        return action::add::Add::new(handler.matches()).invoke();
    }

    #[cfg(feature = "alias")]
    if handler.alias().is_some() {
        return action::alias::Alias::new(handler.matches()).invoke();
    }

    if handler.clone().is_some() {
        return action::clone::Clone::new(handler.matches()).invoke();
    }

    #[cfg(feature = "clipboard")]
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

    if handler.grep().is_some() {
        return action::grep::Grep::new(handler.matches()).invoke();
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

    if handler.internal().is_some() {
        return action::internal::Internal::new(handler.matches()).invoke();
    }

    if handler.list().is_some() {
        return action::list::List::new(handler.matches()).invoke();
    }

    if handler.slam().is_some() {
        return action::slam::Slam::new(handler.matches()).invoke();
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

    #[cfg(all(feature = "tomb", target_os = "linux"))]
    if handler.tomb().is_some() {
        return action::tomb::Tomb::new(handler.matches()).invoke();
    }

    #[cfg(feature = "totp")]
    if handler.totp().is_some() {
        return action::totp::Totp::new(handler.matches()).invoke();
    }

    // Get the main matcher
    let matcher_main = MainMatcher::with(handler.matches()).unwrap();
    if !matcher_main.quiet() {
        print_main_info(&matcher_main);
    }

    Ok(())
}

/// Print the main info, shown when no subcommands were supplied.
pub fn print_main_info(matcher_main: &MainMatcher) -> ! {
    // Get the name of the used executable
    let bin = util::bin_name();

    // Attempt to load default store
    let store = Store::open(matcher_main.store()).ok();
    let has_sync = store.as_ref().map(|s| s.sync().is_init()).unwrap_or(false);

    // Print the main info
    eprintln!("{} {}", crate_name!(), crate_version!());
    eprintln!("Usage: {} [FLAGS] <SUBCOMMAND> ...", bin);
    eprintln!(crate_description!());
    eprintln!();

    if let Some(store) = store {
        #[cfg(not(all(feature = "tomb", target_os = "linux")))]
        let has_closed_tomb = false;
        #[cfg(all(feature = "tomb", target_os = "linux"))]
        let has_closed_tomb = {
            let tomb = store.tomb(
                !matcher_main.verbose(),
                matcher_main.verbose(),
                matcher_main.force(),
            );
            tomb.is_tomb() && !tomb.is_open().unwrap_or(true)
        };

        // Hint tomb open command
        if has_closed_tomb {
            eprintln!("Open password store Tomb:");
            eprintln!("    {}", style::highlight(&format!("{} tomb open", bin)));
            eprintln!();
        }

        // Hint user to add ourselves as recipient if it doesn't have recipient we own
        let we_own_any_recipient = store
            .recipients()
            .and_then(|recip| prs_lib::crypto::recipients::contains_own_secret_key(&recip))
            .unwrap_or(false);
        if !has_closed_tomb && !we_own_any_recipient {
            let config = crate::crypto::config(matcher_main);
            let system_has_secret = prs_lib::crypto::util::has_private_key(&config).unwrap_or(true);
            if system_has_secret {
                eprintln!("Add your own key as recipient or generate a new one:");
            } else {
                eprintln!("Generate and add a new recipient key for yourself:");
            }
            if system_has_secret {
                eprintln!(
                    "    {}",
                    style::highlight(&format!("{} recipients add --secret", bin))
                );
            }
            eprintln!(
                "    {}",
                style::highlight(&format!("{} recipients generate", bin))
            );
            eprintln!();
        }

        // Hint show/copy commands if user has secret
        let has_secret = store.secret_iter().next().is_some();
        if has_closed_tomb || has_secret {
            #[cfg(not(feature = "clipboard"))]
            eprintln!("Show a secret:");
            #[cfg(feature = "clipboard")]
            eprintln!("Show or copy a secret:");
            eprintln!("    {}", style::highlight(&format!("{} show [NAME]", bin)));
            #[cfg(feature = "clipboard")]
            eprintln!("    {}", style::highlight(&format!("{} copy [NAME]", bin)));
            eprintln!();
        }

        // Hint add/edit/remove commands if store has recipient we own
        if has_closed_tomb || we_own_any_recipient {
            eprintln!("Generate, add, edit or remove secrets:");
            eprintln!(
                "    {}",
                style::highlight(&format!("{} generate <NAME>", bin))
            );
            eprintln!("    {}", style::highlight(&format!("{} add <NAME>", bin)));
            eprintln!("    {}", style::highlight(&format!("{} edit [NAME]", bin)));
            eprintln!(
                "    {}",
                style::highlight(&format!("{} remove [NAME]", bin))
            );
            eprintln!();
        }

        // Hint about sync
        if !has_closed_tomb {
            if has_sync {
                eprintln!("Sync your password store:");
                eprintln!("    {}", style::highlight(&format!("{} sync", bin)));
                eprintln!();
            } else {
                eprintln!("Enable sync for your password store:");
                eprintln!("    {}", style::highlight(&format!("{} sync init", bin)));
                eprintln!();
            }
        }
    } else {
        eprintln!("Initialize a new password store or clone an existing one:");
        eprintln!("    {}", style::highlight(&format!("{} init", bin)));
        eprintln!(
            "    {}",
            style::highlight(&format!("{} clone <GIT_URL>", bin))
        );
        eprintln!();
    }

    eprintln!("Show all subcommands, features and other help:");
    eprintln!(
        "    {}",
        style::highlight(&format!("{} help [SUBCOMMAND]", bin))
    );

    quit()
}
