use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{gio, glib};

use crate::config::{APP_ID, PROFILE, VERSION};
use crate::window::LookbookWindow;

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

        self.add_action_entries([quit, about, show_item, can_i_run]);

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
            self.add_action_entries([scroll_to, show_spotlight]);
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
                "Distro, desktop and compositor screenshots load live from <a href=\"https://commons.wikimedia.org\">Wikimedia Commons</a>; none are bundled. Tooltips name each image's author and license, and its Commons page holds the full details.\n\nPackage data in Can I Run It? comes from <a href=\"https://repology.org\">Repology</a>.",
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
