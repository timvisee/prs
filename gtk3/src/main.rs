use std::env::args;

use gio::prelude::*;
use glib::clone;
use gtk::prelude::*;

use prs_lib::store::{Secret, Store};

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
        let model = completion.get_model().unwrap();
        let item = model.get_value(iter, 0);

        // Get item text
        let text: Result<Option<String>, _> = item.get();
        let text = match text {
            Ok(Some(text)) => text,
            _ => return false,
        };

        // Match item text to query
        text.contains(query)
    });

    // Create a ListStore of items
    // These will be the source for the autocompletion
    // as the user types into the field
    // For a more evolved example of ListStore see src/bin/list_store.rs
    let ls = create_list_model(secrets);
    completion.set_model(Some(&ls));

    let input_field = gtk::Entry::new();
    input_field.set_completion(Some(&completion));

    let row = gtk::Box::new(gtk::Orientation::Vertical, 5);
    row.add(&win_title);
    row.pack_start(&input_field, false, false, 10);

    // window.add(&win_title);
    window.add(&row);

    // show everything
    window.show_all();
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
