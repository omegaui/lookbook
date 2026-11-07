use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{gio, glib};

use crate::config::{APP_ID, PROFILE, VERSION};
use crate::window::LookbookWindow;

/// Alt and a number open each page, in the order of the tabs.
const PAGE_SHORTCUTS: [(&str, &str); 6] = [
    ("explore", "<Alt>1"),
    ("distros", "<Alt>2"),
    ("desktops", "<Alt>3"),
    ("compositors", "<Alt>4"),
    ("wallpapers", "<Alt>5"),
    ("system", "<Alt>6"),
];

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct LookbookApplication;

    #[glib::object_subclass]
    impl ObjectSubclass for LookbookApplication {
        const NAME: &'static str = "LookbookApplication";
        type Type = super::LookbookApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for LookbookApplication {}

    impl ApplicationImpl for LookbookApplication {
        fn activate(&self) {
            self.parent_activate();
            let app = self.obj();

            if let Some(window) = app.active_window() {
                window.present();
                return;
            }

            LookbookWindow::new(&app).present();
        }

        fn startup(&self) {
            self.parent_startup();
            let app = self.obj();

            gtk::Window::set_default_icon_name(APP_ID);
            app.setup_actions();
            app.setup_accels();
        }
    }

    impl GtkApplicationImpl for LookbookApplication {}
    impl AdwApplicationImpl for LookbookApplication {}
}

glib::wrapper! {
    pub struct LookbookApplication(ObjectSubclass<imp::LookbookApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl LookbookApplication {
    fn setup_actions(&self) {
        let quit = gio::ActionEntry::builder("quit")
            .activate(|app: &Self, _, _| app.quit())
            .build();
        let about = gio::ActionEntry::builder("about")
            .activate(|app: &Self, _, _| app.show_about_dialog())
            .build();
        let shortcuts = gio::ActionEntry::builder("shortcuts")
            .activate(|app: &Self, _, _| app.show_shortcuts_dialog())
            .build();
        // Opens a details page from outside the window, for example
        // `gapplication action <app-id> show-item "'desktop:gnome'"`.
        let show_item = gio::ActionEntry::builder("show-item")
            .parameter_type(Some(glib::VariantTy::STRING))
            .activate(|app: &Self, _, parameter| app.forward_to_window("show-item", parameter))
            .build();

        let can_i_run = gio::ActionEntry::builder("can-i-run")
            .parameter_type(Some(glib::VariantTy::STRING))
            .activate(|app: &Self, _, parameter| app.forward_to_window("can-i-run", parameter))
            .build();

        self.add_action_entries([quit, about, shortcuts, show_item, can_i_run]);

        if PROFILE == "Devel" {
            // Lets scripts check the lower sections of a page without the mouse.
            let scroll_to = gio::ActionEntry::builder("scroll-to")
                .parameter_type(Some(glib::VariantTy::DOUBLE))
                .activate(|app: &Self, _, parameter| app.forward_to_window("scroll-to", parameter))
                .build();
            let show_spotlight = gio::ActionEntry::builder("show-spotlight")
                .parameter_type(Some(glib::VariantTy::INT32))
                .activate(|app: &Self, _, parameter| app.forward_to_window("show-spotlight", parameter))
                .build();
            // Opens "How Linux Fits Together" on a setup and layer, for example "desktop:2".
            let explain_layers = gio::ActionEntry::builder("explain-layers")
                .parameter_type(Some(glib::VariantTy::STRING))
                .activate(|app: &Self, _, parameter| app.forward_to_window("explain-layers", parameter))
                .build();
            // Main menu items, so scripts can open them without the mouse.
            let what_is_lookbook = gio::ActionEntry::builder("what-is-lookbook")
                .activate(|app: &Self, _, _| app.forward_to_window("what-is-lookbook", None))
                .build();
            let refresh_catalog = gio::ActionEntry::builder("refresh-catalog")
                .activate(|app: &Self, _, _| app.forward_to_window("refresh-catalog", None))
                .build();
            self.add_action_entries([scroll_to, show_spotlight, explain_layers, what_is_lookbook, refresh_catalog]);
        }
    }

    fn forward_to_window(&self, action_name: &str, target: Option<&glib::Variant>) {
        let Some(window) = self.active_window() else {
            return;
        };
        let window_action = format!("win.{action_name}");
        if let Err(error) = window.activate_action(&window_action, target) {
            tracing::warn!("Could not run {window_action}: {error}");
        }
    }

    fn catalog_date(&self) -> Option<String> {
        self.active_window()
            .and_downcast::<LookbookWindow>()
            .and_then(|window| window.catalog_date())
    }

    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Control>q"]);
        self.set_accels_for_action("window.close", &["<Control>w"]);
        self.set_accels_for_action("app.shortcuts", &["<Control>question"]);
        self.set_accels_for_action("win.what-is-lookbook", &["F1"]);
        self.set_accels_for_action("win.refresh-catalog", &["<Control>r"]);
        for (page, key) in PAGE_SHORTCUTS {
            self.set_accels_for_action(&format!("win.show-page('{page}')"), &[key]);
        }
    }

    fn show_shortcuts_dialog(&self) {
        let dialog = adw::ShortcutsDialog::new();
        dialog.add(general_shortcuts());
        dialog.add(page_shortcuts());
        dialog.present(self.active_window().as_ref());
    }

    fn show_about_dialog(&self) {
        let dialog = adw::AboutDialog::builder()
            .application_name("Lookbook")
            .application_icon(APP_ID)
            .developer_name("omegaui")
            .license_type(gtk::License::Gpl30)
            .website("https://github.com/omegaui/lookbook")
            .issue_url("https://github.com/omegaui/lookbook/issues")
            .version(VERSION)
            .build();

        if let Some(date) = self.catalog_date() {
            dialog.set_comments(&gettext("Catalog data from {}").replace("{}", &date));
        }

        dialog.add_legal_section(
            &gettext("Logos"),
            None,
            gtk::License::Custom,
            Some(&gettext(
                "Distro, desktop and compositor logos are trademarks of their owners. Lookbook uses them only to identify each project.\n\nLogos come from <a href=\"https://simpleicons.org\">Simple Icons</a> (CC0 1.0, with these exceptions: Debian CC BY-SA 3.0, Gentoo CC BY-SA 2.5, NixOS CC BY 4.0, niri CC BY-SA 4.0, Fedora under the Fedora logo usage guidelines) and <a href=\"https://github.com/Lukas-W/font-logos\">font-logos</a> (Unlicense) for Budgie, LXQt, MATE and river. The Bazzite logo comes from its press kit (Apache-2.0), the labwc logo is by Johan Malm (CC BY-SA 4.0), and the Wayfire logo is traced from wf-shell's icon (MIT).",
            )),
        );

        dialog.add_legal_section(
            &gettext("Screenshots"),
            None,
            gtk::License::Custom,
            Some(&gettext(
                "Screenshots load live from where they are hosted; none are bundled. Most come from <a href=\"https://commons.wikimedia.org\">Wikimedia Commons</a>, the rest from each project's own website or repository, or from a rice's README. Each one names its author and license, with a link to its source.\n\nThe Hall of Fame shows the winners of <a href=\"https://hypr.land/hall_of_fame/\">Hyprland's ricing contests</a>, with screenshots from hypr.land; each belongs to the rice's author.\n\nPackage data in Can I Run It? comes from <a href=\"https://repology.org\">Repology</a>.",
            )),
        );

        dialog.add_legal_section(
            &gettext("Wayland protocol data"),
            Some("© 2026 Valentin Hăloiu, wayland-explorer"),
            gtk::License::MitX11,
            None,
        );

        dialog.present(self.active_window().as_ref());
    }
}

impl Default for LookbookApplication {
    fn default() -> Self {
        glib::Object::builder()
            .property("application-id", APP_ID)
            .property("resource-base-path", "/io/github/omegaui/Lookbook/")
            .build()
    }
}

fn general_shortcuts() -> adw::ShortcutsSection {
    let section = adw::ShortcutsSection::new(Some(&gettext("General")));
    section.add(adw::ShortcutsItem::from_action(&gettext("What Is Lookbook?"), "win.what-is-lookbook"));
    section.add(adw::ShortcutsItem::from_action(&gettext("Refresh Catalog"), "win.refresh-catalog"));
    section.add(adw::ShortcutsItem::from_action(&gettext("Keyboard Shortcuts"), "app.shortcuts"));
    section.add(adw::ShortcutsItem::from_action(&gettext("Close Window"), "window.close"));
    section.add(adw::ShortcutsItem::from_action(&gettext("Quit"), "app.quit"));
    section
}

fn page_shortcuts() -> adw::ShortcutsSection {
    let section = adw::ShortcutsSection::new(Some(&gettext("Pages")));
    let titles = [
        gettext("Explore"),
        gettext("Distros"),
        gettext("Desktops"),
        gettext("Compositors"),
        gettext("Wallpapers"),
        gettext("Your System"),
    ];
    for (title, (_, key)) in titles.iter().zip(PAGE_SHORTCUTS) {
        section.add(adw::ShortcutsItem::new(title, key));
    }
    section.add(adw::ShortcutsItem::new(&gettext("Go Back"), "<Alt>Left"));
    section
}
