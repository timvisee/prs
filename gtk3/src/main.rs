use std::env::args;
use std::sync::{Arc, Mutex};

use gio::prelude::*;
use glib::clone;
use gtk::prelude::*;

use prs_lib::store::{FindSecret, Secret, Store};

/// Clipboard timeout in seconds.
const TIMEOUT: u32 = 20;

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
    let col_types: [glib::Type; 1] = [glib::Type::String];
    let store = gtk::ListStore::new(&col_types);
    let col_indices: [u32; 1] = [0];
    for d in data.iter() {
        let values: [&dyn ToValue; 1] = [&d.name()];
        store.set(&store.append(), &col_indices, &values);
    }
    store
}

fn build_ui(application: &gtk::Application) {
    // Load secrets from store
    // TODO: show error popup on load failure
    let store = Store::open(prs_lib::STORE_DEFAULT_ROOT).unwrap();
    let secrets = store.secrets(None);

    // TODO: show warning if user has no secrets

    // create the main window
    let window = gtk::ApplicationWindow::new(application);
    window.set_title("prs copy");
    window.set_border_width(5);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(400, 150);

    // Create a title label
    let win_title = gtk::Label::new(None);
    win_title.set_markup("Start typing to select a secret:");

    // Create an EntryCompletion widget
    let completion = gtk::EntryCompletion::new();
    completion.set_text_column(0);
    completion.set_minimum_key_length(1);
    completion.set_popup_completion(true);
    completion.set_inline_completion(true);
    completion.set_inline_selection(true);
    completion.set_match_func(|completion, query, iter| {
        model_item_text(&completion.get_model().unwrap(), iter)
            .map(|text| text.contains(query))
            .unwrap_or(false)
    });

    let ls = create_list_model(secrets);
    completion.set_model(Some(&ls));

    let input_field = gtk::SearchEntry::new();
    input_field.set_completion(Some(&completion));

    // Action handlers to copy selected secret
    let input_field_signal = input_field.clone();
    completion.connect_match_selected(move |_self, _model, _iter| {
        input_field_signal.emit_activate();
        Inhibit(false)
    });
    input_field.connect_activate(move |entry| {
        selected_entry(store.clone(), entry.get_text().into());
    });

    let row = gtk::Box::new(gtk::Orientation::Vertical, 5);
    row.add(&win_title);
    row.pack_start(&input_field, false, false, 10);

    // window.add(&win_title);
    window.add(&row);

    // show everything
    window.show_all();
}

/// Called when we've selected a secret in the input field.
///
/// Shows an error if it doesn't resolve to exactly one.
fn selected_entry(store: Store, query: String) {
    let secret = match store.find(Some(query)) {
        FindSecret::Exact(secret) => secret,
        FindSecret::Many(_) => {
            gtk::MessageDialog::new(
                // TODO: set parent window
                None::<&gtk::Window>,
                gtk::DialogFlags::MODAL,
                gtk::MessageType::Error,
                gtk::ButtonsType::Close,
                "Found multiple secrets for this query. Please refine your query.",
            )
            .show_all();
            return;
        }
    };

    selected(secret);
}

/// Called when we've selected a secret.
///
/// Copies to clipboard with revert timeout.
fn selected(secret: Secret) {
    // TODO: do not unwrap
    // let mut plaintext = prs_lib::crypto::decrypt_file(&secret.path).map_err(Err::Read)?;
    let plaintext = prs_lib::crypto::decrypt_file(&secret.path)
        .unwrap()
        .first_line()
        .unwrap();

    // TODO: do not unwrap
    let text = plaintext.unsecure_to_str().unwrap();

    // Copy with revert timeout
    copy(text.to_string(), TIMEOUT);
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
                    return;
                }
            }

            // Fallback
            clipboard.set_text("");
        });

        Continue(false)
    });
}

/// Get the text for a tree model item by iterator.
fn model_item_text(model: &gtk::TreeModel, iter: &gtk::TreeIter) -> Option<String> {
    let item = model.get_value(iter, 0);

    // Get item text
    let text: Result<Option<String>, _> = item.get();
    match text {
        Ok(Some(text)) => Some(text),
        _ => None,
    }
}

fn main() {
    let application = gtk::Application::new(Some("com.timvisee.prs.gtk3-copy"), Default::default())
        .expect("Initialization failed...");
    application.connect_activate(|app| {
        build_ui(app);
    });

    // When activated, shuts down the application
    let quit = gio::SimpleAction::new("quit", None);
    quit.connect_activate(clone!(@weak application => move |_action, _parameter| {
        application.quit();
    }));
    application.set_accels_for_action("app.quit", &["<Primary>Q"]);
    application.add_action(&quit);

    // Run the application
    application.run(&args().collect::<Vec<_>>());
}
