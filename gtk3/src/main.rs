use std::sync::{Arc, Mutex};

use gio::prelude::*;
#[cfg(all(feature = "notify", not(target_env = "musl")))]
use glib::clone;
use glib::{ControlFlow, Propagation};
use gtk::prelude::*;
#[cfg(all(feature = "notify", target_os = "linux", not(target_env = "musl")))]
use notify_rust::Hint;
#[cfg(all(feature = "notify", not(target_env = "musl")))]
use notify_rust::Notification;

use prs_lib::{
    crypto::{self, prelude::*, Config, Proto},
    store::FindSecret,
    Secret, Store,
};

/// Application ID.
const APP_ID: &str = "com.timvisee.prs.gtk3-copy";

/// Application name.
#[cfg(all(feature = "notify", not(target_env = "musl")))]
const APP_NAME: &str = "prs";

/// Application window title.
const APP_TITLE: &str = "prs quick copy";

/// Default cryptography protocol.
const PROTO: Proto = Proto::Gpg;

/// Clipboard timeout in seconds.
const CLIPBOARD_TIMEOUT: u32 = 20;

fn main() {
    let application = gtk::Application::new(Some(APP_ID), Default::default());
    application.connect_activate(|app| {
        build_ui(app);
    });

    // When activated, shuts down the application
    let quit = gio::SimpleAction::new("quit", None);
    #[cfg(all(feature = "notify", not(target_env = "musl")))]
    quit.connect_activate(clone!(@weak application => move |_action, _parameter| {
        application.quit();
    }));
    application.set_accels_for_action("app.quit", &["Escape"]);
    application.add_action(&quit);

    // Run the application
    application.run();
}

/// Wraps a store secret.
struct Data {
    secret: Secret,
}

impl Data {
    fn name(&self) -> &str {
        &self.secret.name
    }
}

impl From<Secret> for Data {
    fn from(secret: Secret) -> Self {
        Data { secret }
    }
}

/// Create GTK list model for given secrets.
fn create_list_model(secrets: Vec<Secret>) -> gtk::ListStore {
    let data: Vec<Data> = secrets.into_iter().map(|s| s.into()).collect();
    let col_types: [glib::Type; 1] = [glib::Type::STRING];
    let store = gtk::ListStore::new(&col_types);
    for d in data.iter() {
        let values: [(u32, &dyn ToValue); 1] = [(0, &d.name())];
        store.set(&store.append(), &values);
    }
    store
}

fn build_ui(application: &gtk::Application) {
    // Load store
    let store = match Store::open(prs_lib::STORE_DEFAULT_ROOT) {
        Ok(store) => store,
        Err(err) => {
            error_dialog(
                &format!("Failed to load password store.\n\nError: {err}"),
                None,
            );
            application.quit();
            return;
        }
    };
    #[cfg(all(feature = "tomb", target_os = "linux"))]
    let tomb = store.tomb(false, false, true);

    // Prepare tomb
    #[cfg(all(feature = "tomb", target_os = "linux"))]
    if let Err(err) = tomb.prepare() {
        eprintln!("{err}");
        error_dialog("Failed to prepare password store tomb", None);
        application.quit();
        return;
    }

    // Find secrets
    let secrets = store.secrets(None);

    // Quit if user has no secrets
    if secrets.is_empty() {
        error_dialog("Your password store does not have any secrets.", None);
        application.quit();
        return;
    }

    // Create the main window
    let window = gtk::ApplicationWindow::new(application);
    window.set_title(APP_TITLE);
    window.set_border_width(5);
    window.set_position(gtk::WindowPosition::Center);
    window.set_keep_above(true);
    window.set_urgency_hint(true);
    window.set_type_hint(gdk::WindowTypeHint::Dialog);
    window.stick();

    // Create an EntryCompletion widget
    let completion = gtk::EntryCompletion::new();
    completion.set_text_column(0);
    completion.set_minimum_key_length(1);
    completion.set_popup_completion(true);
    completion.set_inline_completion(true);
    completion.set_inline_selection(true);
    completion.set_match_func(|completion, query, iter| {
        model_item_text(&completion.model().unwrap(), iter)
            .map(|text| text.contains(query))
            .unwrap_or(false)
    });

    let ls = create_list_model(secrets);
    completion.set_model(Some(&ls));

    let input_field = gtk::SearchEntry::new();
    input_field.set_completion(Some(&completion));
    input_field.set_width_chars(40);
    input_field.set_placeholder_text(Some("Search for a secret..."));
    input_field.set_input_hints(gtk::InputHints::NO_SPELLCHECK);

    // Action handlers to copy selected secret
    let input_field_signal = input_field.clone();
    completion.connect_match_selected(move |_self, _model, _iter| {
        input_field_signal.emit_activate();
        Propagation::Stop
    });

    let window_ref = window.clone();
    let input_ref = input_field.clone();
    input_field.connect_activate(move |entry| {
        selected_entry(
            store.clone(),
            entry.text().into(),
            window_ref.clone(),
            input_ref.clone(),
        );
    });

    window.add(&input_field);

    // show everything
    window.show_all();
    window.grab_focus();

    // TODO: finalize store tomb somewhere
}

/// Called when we've selected a secret in the input field.
///
/// Shows an error if it doesn't resolve to exactly one.
fn selected_entry(
    store: Store,
    query: String,
    window: gtk::ApplicationWindow,
    input: gtk::SearchEntry,
) {
    // Show error for empty query
    if query.trim().is_empty() {
        error_dialog("Please enter the name of a secret to copy.", Some(&window));
        return;
    }

    let secret = match store.find(Some(query)) {
        FindSecret::Exact(secret) => secret,
        FindSecret::Many(secrets) if secrets.len() == 1 => secrets[0].clone(),
        FindSecret::Many(secrets) if secrets.is_empty() => {
            error_dialog(
                "Found no secrets for this query. Please name a specific secret.",
                Some(&window),
            );
            return;
        }
        FindSecret::Many(secrets) => {
            error_dialog(
                &format!(
                    "Found {} secrets for this query. Please refine your query.",
                    secrets.len()
                ),
                Some(&window),
            );
            return;
        }
    };

    selected(secret, window, input);
}

/// Called when we've selected a secret.
///
/// Copies to clipboard with revert timeout.
fn selected(secret: Secret, window: gtk::ApplicationWindow, input: gtk::SearchEntry) {
    // Decrypt first line of plaintext
    let config = Config::from(PROTO);
    let plaintext = match crypto::context(&config)
        .map_err(|err| err.into())
        .and_then(|mut context| context.decrypt_file(&secret.path))
        .and_then(|plaintext| plaintext.first_line())
    {
        Ok(plaintext) => plaintext,
        Err(err) => {
            error_dialog(
                &format!("Failed to decrypt first line of secret.\n\nError: {err}"),
                Some(&window),
            );
            window.close();
            return;
        }
    };

    let text = plaintext.unsecure_to_str().unwrap();

    // Copy with revert timeout
    copy(text.to_string(), CLIPBOARD_TIMEOUT);

    // Move to back, disable input
    window.set_keep_above(false);
    window.set_sensitive(false);
    window.set_deletable(false);
    window.unstick();
    input.set_text("");
    input.set_placeholder_text(Some(&format!(
        "Copied, clearing in {CLIPBOARD_TIMEOUT} seconds...",
    )));

    // Hack to unfocus and move window to back
    window.set_accept_focus(false);
    window.set_focus(None::<&gtk::Widget>);
    if let Some(window) = window.window() {
        window.hide();
        window.show_unraised();
        window.lower();
    }

    // Close window after clipboard revert
    // TODO: wait for clipboard revert instead, do not use own timeout
    glib::timeout_add_seconds_local(CLIPBOARD_TIMEOUT + 1, move || {
        window.close();
        ControlFlow::Continue
    });
}

/// Copy given text to clipboard with revert timeout.
fn copy(text: String, timeout: u32) {
    // Get clipboard context
    let clipboard = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);

    // Obtain previous clipboard contents
    let previous = Arc::new(Mutex::new(None));
    let previous_clone = previous.clone();
    clipboard.request_text(move |_clipboard, text| {
        if let Ok(mut previous) = previous_clone.lock() {
            *previous = text.map(|t| t.to_string());
        }
    });

    clipboard.set_text(&text);

    // Wait for timeout, then revert clipboard
    glib::timeout_add_seconds_local(timeout, move || {
        let previous = previous.clone();
        let text = text.clone();

        // Obtain current clipboard contents, change to previous if secret is still in it
        clipboard.request_text(move |clipboard, current| {
            if current != Some(&text) {
                return;
            }

            // Set to previous if secret is still in
            if let Ok(previous) = previous.lock() {
                if let Some(ref previous) = *previous {
                    clipboard.set_text(previous);
                    notify_cleared();
                    return;
                }
            }

            // Fallback
            clipboard.set_text("");
            notify_cleared();
        });

        ControlFlow::Continue
    });
}

/// Show notification to user about cleared clipboard.
fn notify_cleared() {
    // Do not show notification with not notify or on musl due to segfault
    #[cfg(all(feature = "notify", not(target_env = "musl")))]
    {
        let mut n = Notification::new();
        n.appname(APP_NAME)
            .summary(&format!("Clipboard cleared - {APP_NAME}"))
            .body("Secret cleared from clipboard")
            .auto_icon()
            .icon("lock")
            .timeout(3000);

        #[cfg(target_os = "linux")]
        n.urgency(notify_rust::Urgency::Low)
            .hint(Hint::Category("presence.offline".into()));

        let _ = n.show();
        return;
    }

    // Fallback if we cannot notify
    #[cfg_attr(
        all(feature = "notify", not(target_env = "musl")),
        expect(unreachable_code)
    )]
    {
        eprintln!("Secret cleared from clipboard");
    }
}

/// Show an error dialog.
fn error_dialog(msg: &str, window: Option<&gtk::ApplicationWindow>) {
    let dialog = gtk::MessageDialog::new(
        window,
        gtk::DialogFlags::MODAL,
        gtk::MessageType::Error,
        gtk::ButtonsType::Close,
        msg,
    );
    dialog.connect_response(|dialog, _response| dialog.close());
    dialog.run();
}

/// Get the text for a tree model item by iterator.
fn model_item_text(model: &gtk::TreeModel, iter: &gtk::TreeIter) -> Option<String> {
    let item = model.value(iter, 0);

    // Get item text
    let text: Result<Option<String>, _> = item.get();
    match text {
        Ok(Some(text)) => Some(text),
        _ => None,
    }
}
