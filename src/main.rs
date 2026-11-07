mod application;
mod catalog;
#[rustfmt::skip]
mod config;
mod details;
mod extensions;
mod http;
mod ideas;
mod pages;
mod repology;
mod run_check;
mod system;
mod wallpapers;
mod widgets;
mod window;

use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
use gtk::prelude::*;
use gtk::{gio, glib};

use self::application::LinuxAndYouApplication;
use self::config::{GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE};

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();

    setup_translations();
    load_resources();
    glib::set_application_name("Linux & You");

    let app = LinuxAndYouApplication::default();
    app.run()
}

fn setup_translations() {
    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to set the text domain encoding");
    textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");
}

fn load_resources() {
    let resources = gio::Resource::load(RESOURCES_FILE).expect("Could not load resources");
    gio::resources_register(&resources);
}
