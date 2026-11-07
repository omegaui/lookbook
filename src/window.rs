use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::application::LinuxAndYouApplication;
use crate::catalog::{Catalog, ItemRef};
use crate::config::{APP_ID, PROFILE};
use crate::pages::{DistrosPage, ExplorePage, ItemDetailPage};
use crate::system::SystemInfo;
use crate::widgets::DetailContent;

const MAIN_PAGE_TAG: &str = "main";

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/LinuxAndYou/ui/window.ui")]
    pub struct LinuxAndYouWindow {
        #[template_child]
        pub navigation_view: gtk::TemplateChild<adw::NavigationView>,
        #[template_child]
        pub view_stack: gtk::TemplateChild<adw::ViewStack>,
        #[template_child]
        pub explore_page: gtk::TemplateChild<ExplorePage>,
        #[template_child]
        pub distros_page: gtk::TemplateChild<DistrosPage>,
        pub settings: gio::Settings,
        pub catalog: RefCell<Option<Rc<Catalog>>>,
        pub system: RefCell<Option<Rc<SystemInfo>>>,
    }

    impl Default for LinuxAndYouWindow {
        fn default() -> Self {
            Self {
                navigation_view: gtk::TemplateChild::default(),
                view_stack: gtk::TemplateChild::default(),
                explore_page: gtk::TemplateChild::default(),
                distros_page: gtk::TemplateChild::default(),
                settings: gio::Settings::new(APP_ID),
                catalog: RefCell::default(),
                system: RefCell::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LinuxAndYouWindow {
        const NAME: &'static str = "LinuxAndYouWindow";
        type Type = super::LinuxAndYouWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            ExplorePage::ensure_type();
            DistrosPage::ensure_type();
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LinuxAndYouWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let window = self.obj();

            window.mark_development_build();
            window.load_window_state();
            window.setup_actions();
            window.remember_last_page();
            window.load_catalog();
        }
    }

    impl WidgetImpl for LinuxAndYouWindow {}

    impl WindowImpl for LinuxAndYouWindow {
        fn close_request(&self) -> glib::Propagation {
            if let Err(error) = self.obj().save_window_state() {
                tracing::warn!("Could not save the window state: {error}");
            }
            self.parent_close_request()
        }
    }

    impl ApplicationWindowImpl for LinuxAndYouWindow {}
    impl AdwApplicationWindowImpl for LinuxAndYouWindow {}
}

glib::wrapper! {
    pub struct LinuxAndYouWindow(ObjectSubclass<imp::LinuxAndYouWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl LinuxAndYouWindow {
    pub fn new(app: &LinuxAndYouApplication) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    /// Development builds get the striped header bar, so they are easy to tell apart.
    fn mark_development_build(&self) {
        if PROFILE == "Devel" {
            self.add_css_class("devel");
        }
    }

    /// `win.show-page` switches to a page by name, for example "system".
    /// `win.show-item` opens the details of an item, for example "distro:fedora".
    fn setup_actions(&self) {
        let show_page = gio::ActionEntry::builder("show-page")
            .parameter_type(Some(glib::VariantTy::STRING))
            .activate(|window: &Self, _, parameter| {
                if let Some(page_name) = parameter.and_then(|value| value.get::<String>()) {
                    window.show_page(&page_name);
                }
            })
            .build();

        let show_item = gio::ActionEntry::builder("show-item")
            .parameter_type(Some(glib::VariantTy::STRING))
            .activate(|window: &Self, _, parameter| {
                let target = parameter.and_then(|value| value.get::<String>());
                if let Some(item) = target.as_deref().and_then(ItemRef::parse) {
                    window.show_item(&item);
                }
            })
            .build();

        self.add_action_entries([show_page, show_item]);
    }

    fn show_page(&self, page_name: &str) {
        let imp = self.imp();
        imp.navigation_view.pop_to_tag(MAIN_PAGE_TAG);
        imp.view_stack.set_visible_child_name(page_name);
    }

    fn show_item(&self, item: &ItemRef) {
        let imp = self.imp();
        let catalog = imp.catalog.borrow().clone();
        let system = imp.system.borrow().clone();
        let (Some(catalog), Some(system)) = (catalog, system) else {
            return;
        };

        let Some(content) = DetailContent::from_catalog(item, &catalog, &system) else {
            tracing::warn!("The catalog has no {}", item.to_action_target());
            return;
        };

        let screenshots = catalog.screenshots_of(&item.id);
        imp.navigation_view
            .push(&ItemDetailPage::new(&content, &screenshots));
    }

    fn remember_last_page(&self) {
        let imp = self.imp();
        imp.settings
            .bind("last-page", &*imp.view_stack, "visible-child-name")
            .build();
    }

    fn load_catalog(&self) {
        let imp = self.imp();
        let system = Rc::new(SystemInfo::detect());

        let catalog = match Catalog::load_bundled() {
            Ok(catalog) => Rc::new(catalog),
            Err(error) => {
                tracing::error!("Could not load the bundled catalog: {error:#}");
                return;
            }
        };

        imp.explore_page.show_catalog(&catalog, &system);
        imp.distros_page.show_catalog(&catalog, &system);

        imp.catalog.replace(Some(catalog));
        imp.system.replace(Some(system));
    }

    fn load_window_state(&self) {
        let settings = &self.imp().settings;
        let width = settings.int("window-width");
        let height = settings.int("window-height");

        self.set_default_size(width, height);

        if settings.boolean("is-maximized") {
            self.maximize();
        }
    }

    fn save_window_state(&self) -> Result<(), glib::BoolError> {
        let settings = &self.imp().settings;
        let (width, height) = self.default_size();

        settings.set_int("window-width", width)?;
        settings.set_int("window-height", height)?;
        settings.set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }
}
