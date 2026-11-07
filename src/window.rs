use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::application::LinuxAndYouApplication;
use crate::catalog::{Catalog, ItemRef};
use crate::config::{APP_ID, PROFILE};
use crate::pages::{
    CompositorsPage, DesktopsPage, DistrosPage, ExplorePage, ItemDetailPage, SystemPage,
    WallpapersPage,
};
use crate::system::SystemInfo;
use crate::widgets::DetailContent;

const MAIN_PAGE_TAG: &str = "main";
/// Desktops that place windows freely. Tiling compositors size windows themselves
/// and may ignore a maximize request, which leaves the app drawn wider than its tile.
const STACKING_DESKTOPS: [&str; 7] = ["GNOME", "KDE", "X-Cinnamon", "XFCE", "MATE", "Pantheon", "Budgie"];

fn runs_on_stacking_desktop() -> bool {
    let session = crate::system::Session::detect();
    session.desktop_names.iter().any(|running| {
        STACKING_DESKTOPS
            .iter()
            .any(|desktop| desktop.eq_ignore_ascii_case(running))
    })
}

/// Finds the first scrolled window inside a widget, searching depth first.
fn find_scrolled_window(widget: &gtk::Widget) -> Option<gtk::ScrolledWindow> {
    if let Some(scrolled) = widget.downcast_ref::<gtk::ScrolledWindow>() {
        return Some(scrolled.clone());
    }
    let mut child = widget.first_child();
    while let Some(current) = child {
        if let Some(found) = find_scrolled_window(&current) {
            return Some(found);
        }
        child = current.next_sibling();
    }
    None
}

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
        #[template_child]
        pub desktops_page: gtk::TemplateChild<DesktopsPage>,
        #[template_child]
        pub compositors_page: gtk::TemplateChild<CompositorsPage>,
        #[template_child]
        pub wallpapers_page: gtk::TemplateChild<WallpapersPage>,
        #[template_child]
        pub system_page: gtk::TemplateChild<SystemPage>,
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
                desktops_page: gtk::TemplateChild::default(),
                compositors_page: gtk::TemplateChild::default(),
                wallpapers_page: gtk::TemplateChild::default(),
                system_page: gtk::TemplateChild::default(),
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
            DesktopsPage::ensure_type();
            CompositorsPage::ensure_type();
            WallpapersPage::ensure_type();
            SystemPage::ensure_type();
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

        // Until the Can I Run It sheet exists, the question leads to Your System.
        let can_i_run = gio::ActionEntry::builder("can-i-run")
            .parameter_type(Some(glib::VariantTy::STRING))
            .activate(|window: &Self, _, _| window.show_page("system"))
            .build();

        self.add_action_entries([show_page, show_item, can_i_run]);

        if PROFILE == "Devel" {
            self.add_scroll_action();
        }
    }

    /// Development builds only: `win.scroll-to` scrolls the page in view to a
    /// fraction of its height (0.0 is the top, 1.0 the bottom), so a script can
    /// check the lower sections of a page without touching the mouse.
    fn add_scroll_action(&self) {
        let scroll_to = gio::ActionEntry::builder("scroll-to")
            .parameter_type(Some(glib::VariantTy::DOUBLE))
            .activate(|window: &Self, _, parameter| {
                if let Some(fraction) = parameter.and_then(|value| value.get::<f64>()) {
                    window.scroll_to(fraction);
                }
            })
            .build();
        // Turns the Explore spotlight carousel to a page, counting from 0.
        let show_spotlight = gio::ActionEntry::builder("show-spotlight")
            .parameter_type(Some(glib::VariantTy::INT32))
            .activate(|window: &Self, _, parameter| {
                if let Some(index) = parameter.and_then(|value| value.get::<i32>()) {
                    window.imp().explore_page.show_spotlight(index);
                }
            })
            .build();
        self.add_action_entries([scroll_to, show_spotlight]);
    }

    fn scroll_to(&self, fraction: f64) {
        let Some(scrolled) = self.visible_scrolled_window() else {
            return;
        };
        let adjustment = scrolled.vadjustment();
        let range = adjustment.upper() - adjustment.page_size();
        adjustment.set_value(range * fraction.clamp(0.0, 1.0));
    }

    fn visible_scrolled_window(&self) -> Option<gtk::ScrolledWindow> {
        let imp = self.imp();
        let page = imp.navigation_view.visible_page()?;
        let root: gtk::Widget = if page.tag().as_deref() == Some(MAIN_PAGE_TAG) {
            imp.view_stack.visible_child()?
        } else {
            page.upcast()
        };
        find_scrolled_window(&root)
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

        imp.navigation_view
            .push(&ItemDetailPage::new(&content, &catalog, &system));
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
        imp.desktops_page.show_catalog(&catalog, &system);
        imp.compositors_page.show_catalog(&catalog, &system);
        imp.wallpapers_page.show_catalog(&catalog, &system);
        imp.system_page.show_system(catalog.clone(), &system);

        imp.catalog.replace(Some(catalog));
        imp.system.replace(Some(system));
    }

    fn load_window_state(&self) {
        let settings = &self.imp().settings;
        let width = settings.int("window-width");
        let height = settings.int("window-height");

        self.set_default_size(width, height);

        if settings.boolean("is-maximized") && runs_on_stacking_desktop() {
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
