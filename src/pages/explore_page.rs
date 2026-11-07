//! The home page: collections, spotlights, the Hall of fame and the galleries.

use std::cell::{Cell, RefCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::glib::clone;

use gettextrs::gettext;

use crate::catalog::{Catalog, DistroFamily};
use crate::http::shared_session;
use crate::news::{fetch_releases, recent_releases, Day, Release};
use crate::system::SystemInfo;
use crate::widgets::{
    fill_wall_grid, open_details_on_activate, release_card, CardContent, CatalogCard,
    CollectionCard, CollectionContent, SpotlightCard, SpotlightContent, WallpaperSpotlight,
};

/// How many desktops and compositors the home page shows. "See all" opens the rest.
const HOME_DESKTOP_COUNT: usize = 4;
const HOME_COMPOSITOR_COUNT: usize = 4;
const ALL_FAMILIES: &str = "all";
/// "What's new" shows at most three rows of four.
const NEWS_COUNT: usize = 12;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Lookbook/ui/pages/explore_page.ui")]
    pub struct ExplorePage {
        #[template_child]
        pub collections_box: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub spotlight_carousel: gtk::TemplateChild<adw::Carousel>,
        #[template_child]
        pub news_spinner: gtk::TemplateChild<adw::Spinner>,
        #[template_child]
        pub news_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub news_status_label: gtk::TemplateChild<gtk::Label>,
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
        /// Releases found so far, and how many feeds have not answered yet.
        pub releases: RefCell<Vec<Release>>,
        pub feeds_pending: Cell<usize>,
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
            page.load_wallpaper_spotlight_when_shown();
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
        self.show_wallpaper_spotlight(catalog, system);
        self.load_news(catalog);
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

    /// The wallpaper spotlight comes last, after the distro, desktop and compositor.
    fn show_wallpaper_spotlight(&self, catalog: &Catalog, system: &SystemInfo) {
        let Some(spotlight) = &catalog.wallpaper_spotlight else {
            return;
        };
        if let Some(card) = WallpaperSpotlight::new(spotlight, catalog, system) {
            self.imp().spotlight_carousel.append(&card);
        }
    }

    /// The wallpaper is read from its source only once the carousel reaches it,
    /// so opening the app does not download anything the user never sees.
    fn load_wallpaper_spotlight_when_shown(&self) {
        let carousel = &self.imp().spotlight_carousel;
        carousel.connect_page_changed(|carousel, index| {
            let page = carousel.nth_page(index);
            if let Some(card) = page.downcast_ref::<WallpaperSpotlight>() {
                card.load();
            }
        });
    }

    /// Reads every release feed at once; cards appear as feeds answer.
    fn load_news(&self, catalog: &Catalog) {
        let imp = self.imp();
        imp.feeds_pending.set(catalog.news_sources.len());
        let today = today();

        for source in catalog.news_sources.clone() {
            glib::spawn_future_local(clone!(
                #[weak(rename_to = page)]
                self,
                async move {
                    let releases = match fetch_releases(&shared_session(), &source).await {
                        Ok(releases) => recent_releases(&source, releases, today),
                        Err(error) => {
                            tracing::warn!("Could not read {}'s releases: {error:#}", source.name);
                            Vec::new()
                        }
                    };
                    page.add_releases(releases);
                }
            ));
        }
        self.show_news_status();
    }

    fn add_releases(&self, releases: Vec<Release>) {
        let imp = self.imp();
        imp.releases.borrow_mut().extend(releases);
        imp.feeds_pending.set(imp.feeds_pending.get().saturating_sub(1));
        self.show_newest_releases();
        self.show_news_status();
    }

    /// Newest first, so the order settles as slower feeds answer.
    fn show_newest_releases(&self) {
        let imp = self.imp();
        let mut releases = imp.releases.borrow().clone();
        releases.sort_by(|first, second| second.day.cmp(&first.day));

        imp.news_flowbox.remove_all();
        for release in releases.iter().take(NEWS_COUNT) {
            imp.news_flowbox.append(&release_card(release));
        }
    }

    fn show_news_status(&self) {
        let imp = self.imp();
        let still_loading = imp.feeds_pending.get() > 0;
        let nothing_found = imp.releases.borrow().is_empty();

        imp.news_spinner.set_visible(still_loading);
        imp.news_status_label.set_visible(!still_loading && nothing_found);
        imp.news_status_label
            .set_label(&gettext("No new releases reached Lookbook this month. Check your connection, or come back in a few days."));
    }

    fn show_wall_of_fame(&self, catalog: &Catalog) {
        let screenshots: Vec<_> = catalog.screenshots.iter().collect();
        fill_wall_grid(&self.imp().wall_grid, &screenshots, catalog);
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

    /// Turns the carousel straight to page `index`, if it exists.
    pub fn show_spotlight(&self, index: i32) {
        let carousel = &self.imp().spotlight_carousel;
        if index < 0 || index as u32 >= carousel.n_pages() {
            return;
        }
        carousel.scroll_to(&carousel.nth_page(index as u32), false);
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

/// Today's date on the user's clock.
fn today() -> Day {
    let now = glib::DateTime::now_local().expect("the system clock should give a local time");
    Day {
        year: now.year(),
        month: now.month() as u32,
        day: now.day_of_month() as u32,
    }
}
