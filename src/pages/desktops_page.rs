//! The desktop environment gallery. GNOME is featured, because Linux & You is a GNOME app first.

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

use crate::catalog::Catalog;
use crate::system::SystemInfo;
use crate::widgets::{open_details_on_activate, CardContent, CatalogCard, LogoTile, TileContent};

const FEATURED_DESKTOP: &str = "gnome";
const FEATURED_TILE_SIZE: i32 = 96;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/LinuxAndYou/ui/pages/desktops_page.ui")]
    pub struct DesktopsPage {
        #[template_child]
        pub featured_banner: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub featured_tile: gtk::TemplateChild<LogoTile>,
        #[template_child]
        pub featured_name_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub featured_tagline_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub desktops_flowbox: gtk::TemplateChild<gtk::FlowBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DesktopsPage {
        const NAME: &'static str = "DesktopsPage";
        type Type = super::DesktopsPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            LogoTile::ensure_type();
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DesktopsPage {
        fn constructed(&self) {
            self.parent_constructed();
            open_details_on_activate(&self.desktops_flowbox);
        }
    }

    impl WidgetImpl for DesktopsPage {}
    impl BinImpl for DesktopsPage {}
}

glib::wrapper! {
    pub struct DesktopsPage(ObjectSubclass<imp::DesktopsPage>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl DesktopsPage {
    pub fn show_catalog(&self, catalog: &Catalog, system: &SystemInfo) {
        self.show_featured_desktop(catalog);
        self.show_desktops(catalog, system);
    }

    /// Hides the banner if the catalog ever stops listing GNOME.
    fn show_featured_desktop(&self, catalog: &Catalog) {
        let imp = self.imp();
        let Some(desktop) = catalog.find_desktop(FEATURED_DESKTOP) else {
            imp.featured_banner.set_visible(false);
            return;
        };

        imp.featured_tile.set_tile_size(FEATURED_TILE_SIZE);
        imp.featured_tile.set_content(&TileContent {
            short: desktop.short.clone(),
            color: desktop.color.clone(),
            logo: desktop.logo.clone(),
            icon_name: None,
        });
        imp.featured_name_label.set_label(&desktop.name);
        imp.featured_tagline_label.set_label(&desktop.tagline);
    }

    fn show_desktops(&self, catalog: &Catalog, system: &SystemInfo) {
        for desktop in &catalog.desktops {
            let content = CardContent::for_desktop(desktop, system);
            self.imp().desktops_flowbox.append(&CatalogCard::new(&content));
        }
    }
}
