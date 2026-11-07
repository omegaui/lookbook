//! The home page: collections, spotlights, the Hall of fame and the galleries.

use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::glib::clone;

use crate::catalog::{Catalog, DistroFamily};
use crate::system::SystemInfo;
use crate::widgets::{
    fill_wall_grid, open_details_on_activate, CardContent, CatalogCard, CollectionCard,
    CollectionContent, SpotlightCard, SpotlightContent,
};

/// How many desktops and compositors the home page shows. "See all" opens the rest.
const HOME_DESKTOP_COUNT: usize = 4;
const HOME_COMPOSITOR_COUNT: usize = 4;
const ALL_FAMILIES: &str = "all";

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/LinuxAndYou/ui/pages/explore_page.ui")]
    pub struct ExplorePage {
        #[template_child]
        pub collections_box: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub spotlight_carousel: gtk::TemplateChild<adw::Carousel>,
        #[template_child]
        pub wall_grid: gtk::TemplateChild<gtk::Grid>,
        #[template_child]
        pub family_toggles: gtk::TemplateChild<adw::ToggleGroup>,
        #[template_child]
        pub distros_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub desktops_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub compositors_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub system_title_label: gtk::TemplateChild<gtk::Label>,
        /// The family of each card in `distros_flowbox`, in the same order.
        pub distro_families: RefCell<Vec<DistroFamily>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ExplorePage {
        const NAME: &'static str = "ExplorePage";
        type Type = super::ExplorePage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ExplorePage {
        fn constructed(&self) {
            self.parent_constructed();
            let page = self.obj();

            page.setup_distro_filter();
            open_details_on_activate(&self.distros_flowbox);
            open_details_on_activate(&self.desktops_flowbox);
            open_details_on_activate(&self.compositors_flowbox);
        }
    }

    impl WidgetImpl for ExplorePage {}
    impl BinImpl for ExplorePage {}

    #[gtk::template_callbacks]
    impl ExplorePage {
        #[template_callback]
        fn on_previous_spotlight(&self, _button: &gtk::Button) {
            self.obj().scroll_spotlight(-1);
        }

        #[template_callback]
        fn on_next_spotlight(&self, _button: &gtk::Button) {
            self.obj().scroll_spotlight(1);
        }
    }
}

glib::wrapper! {
    pub struct ExplorePage(ObjectSubclass<imp::ExplorePage>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ExplorePage {
    pub fn show_catalog(&self, catalog: &Catalog, system: &SystemInfo) {
        self.show_collections(catalog);
        self.show_spotlights(catalog);
        self.show_wall_of_fame(catalog);
        self.show_distros(catalog, system);
        self.show_desktops(catalog, system);
        self.show_compositors(catalog, system);
        self.show_system_summary(catalog, system);
    }

    fn show_collections(&self, catalog: &Catalog) {
        for collection in &catalog.collections {
            let content = CollectionContent::from_catalog(collection, catalog);
            self.imp().collections_box.append(&CollectionCard::new(&content));
        }
    }

    fn show_spotlights(&self, catalog: &Catalog) {
        for spotlight in &catalog.spotlights {
            if let Some(content) = SpotlightContent::from_catalog(spotlight, catalog) {
                self.imp().spotlight_carousel.append(&SpotlightCard::new(&content));
            }
        }
    }

    fn show_wall_of_fame(&self, catalog: &Catalog) {
        let screenshots: Vec<_> = catalog.screenshots.iter().collect();
        fill_wall_grid(&self.imp().wall_grid, &screenshots);
    }

    fn show_distros(&self, catalog: &Catalog, system: &SystemInfo) {
        let imp = self.imp();

        for distro in &catalog.distros {
            let content = CardContent::for_distro(distro, system);
            imp.distros_flowbox.append(&CatalogCard::new(&content));
        }

        let families = catalog.distros.iter().map(|distro| distro.family).collect();
        imp.distro_families.replace(families);
    }

    fn show_desktops(&self, catalog: &Catalog, system: &SystemInfo) {
        for desktop in catalog.desktops.iter().take(HOME_DESKTOP_COUNT) {
            let content = CardContent::for_desktop(desktop, system);
            self.imp().desktops_flowbox.append(&CatalogCard::new(&content));
        }
    }

    fn show_compositors(&self, catalog: &Catalog, system: &SystemInfo) {
        for compositor in catalog.compositors.iter().take(HOME_COMPOSITOR_COUNT) {
            let content = CardContent::for_compositor(compositor, system);
            self.imp().compositors_flowbox.append(&CatalogCard::new(&content));
        }
    }

    /// Shows "<distro> + <desktop>" in the Your system banner.
    fn show_system_summary(&self, catalog: &Catalog, system: &SystemInfo) {
        let distro = system.distro_name();
        let desktop = system.desktop_name(catalog);
        self.imp()
            .system_title_label
            .set_label(&format!("{distro} + {desktop}"));
    }

    fn setup_distro_filter(&self) {
        let imp = self.imp();

        imp.distros_flowbox.set_filter_func(clone!(
            #[weak(rename_to = page)]
            self,
            #[upgrade_or]
            true,
            move |child| page.distro_matches_filter(child)
        ));

        imp.family_toggles.connect_active_name_notify(clone!(
            #[weak(rename_to = page)]
            self,
            move |_| page.imp().distros_flowbox.invalidate_filter()
        ));
    }

    fn distro_matches_filter(&self, child: &gtk::FlowBoxChild) -> bool {
        let imp = self.imp();
        let Some(active) = imp.family_toggles.active_name() else {
            return true;
        };
        if active.as_str() == ALL_FAMILIES {
            return true;
        }

        let families = imp.distro_families.borrow();
        match families.get(child.index() as usize) {
            Some(family) => family.id() == active.as_str(),
            None => true,
        }
    }

    /// Moves the spotlight carousel by `step` pages, wrapping around at the ends.
    fn scroll_spotlight(&self, step: i32) {
        let carousel = &self.imp().spotlight_carousel;
        let count = carousel.n_pages() as i32;
        if count == 0 {
            return;
        }

        let current = carousel.position().round() as i32;
        let next = (current + step).rem_euclid(count);
        carousel.scroll_to(&carousel.nth_page(next as u32), true);
    }
}
