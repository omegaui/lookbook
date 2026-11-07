//! Wallpapers read live from each collection, ready to set or save.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::glib;
use gtk::glib::clone;

use crate::catalog::{Catalog, WallpaperSource};
use crate::system::SystemInfo;
use crate::wallpapers::{fetch_source, new_session, Wallpaper};
use crate::widgets::{WallpaperCard, WallpaperDialog, WallpaperDialogContext};

/// Thumbnails load in pages, so opening the section stays light on the network.
const PAGE_SIZE: usize = 48;
/// At most this many thumbnails download at once; the rest wait in grid order.
const PARALLEL_THUMBNAILS: usize = 4;
const SHOW_ALL: &str = "all";
const WALLPAPER_PORTAL: &str = "Wallpaper";

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/LinuxAndYou/ui/pages/wallpapers_page.ui")]
    pub struct WallpapersPage {
        #[template_child]
        pub source_toggles: gtk::TemplateChild<adw::ToggleGroup>,
        #[template_child]
        pub content_stack: gtk::TemplateChild<gtk::Stack>,
        #[template_child]
        pub wallpapers_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub show_more_button: gtk::TemplateChild<gtk::Button>,
        #[template_child]
        pub credits_box: gtk::TemplateChild<gtk::Box>,
        pub session: soup::Session,
        pub sources: RefCell<Vec<WallpaperSource>>,
        pub blocklist: RefCell<Vec<String>>,
        /// Shown in the preview when the user's portal may not set wallpapers.
        pub portal_hint: RefCell<Option<String>>,
        /// Every wallpaper read so far, from all sources.
        pub wallpapers: RefCell<Vec<Wallpaper>>,
        /// How many of the filtered wallpapers the grid shows.
        pub shown_count: Cell<usize>,
        pub has_loaded: Cell<bool>,
        /// Cards still waiting for their thumbnail, in grid order.
        pub thumbnail_queue: RefCell<VecDeque<glib::WeakRef<WallpaperCard>>>,
        pub thumbnails_loading: Cell<usize>,
    }

    impl Default for WallpapersPage {
        fn default() -> Self {
            Self {
                source_toggles: gtk::TemplateChild::default(),
                content_stack: gtk::TemplateChild::default(),
                wallpapers_flowbox: gtk::TemplateChild::default(),
                show_more_button: gtk::TemplateChild::default(),
                credits_box: gtk::TemplateChild::default(),
                session: new_session(),
                sources: RefCell::default(),
                blocklist: RefCell::default(),
                portal_hint: RefCell::default(),
                wallpapers: RefCell::default(),
                shown_count: Cell::new(0),
                has_loaded: Cell::new(false),
                thumbnail_queue: RefCell::default(),
                thumbnails_loading: Cell::new(0),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WallpapersPage {
        const NAME: &'static str = "WallpapersPage";
        type Type = super::WallpapersPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for WallpapersPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_page();
        }
    }

    impl WidgetImpl for WallpapersPage {}
    impl BinImpl for WallpapersPage {}

    #[gtk::template_callbacks]
    impl WallpapersPage {
        #[template_callback]
        fn on_retry_clicked(&self, _button: &gtk::Button) {
            self.obj().load_wallpapers();
        }

        #[template_callback]
        fn on_show_more_clicked(&self, _button: &gtk::Button) {
            self.obj().show_next_page();
        }
    }
}

glib::wrapper! {
    pub struct WallpapersPage(ObjectSubclass<imp::WallpapersPage>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl WallpapersPage {
    /// Remembers the sources. Nothing is downloaded until the page is opened.
    pub fn show_catalog(&self, catalog: &Catalog, system: &SystemInfo) {
        let imp = self.imp();
        imp.sources.replace(catalog.wallpaper_sources.clone());
        imp.blocklist.replace(catalog.wallpaper_blocklist.clone());
        imp.portal_hint.replace(portal_hint(catalog, system));

        self.add_source_toggles();
        self.show_credits();
    }

    fn setup_page(&self) {
        let imp = self.imp();

        // Load on first view, not at app start.
        self.connect_map(|page| {
            if !page.imp().has_loaded.get() {
                page.load_wallpapers();
            }
        });

        imp.source_toggles.connect_active_name_notify(clone!(
            #[weak(rename_to = page)]
            self,
            move |_| page.show_first_page()
        ));
    }

    fn add_source_toggles(&self) {
        let toggles = &self.imp().source_toggles;
        toggles.add(adw::Toggle::builder().name(SHOW_ALL).label(gettext("All")).build());

        for source in self.imp().sources.borrow().iter() {
            toggles.add(adw::Toggle::builder().name(&source.id).label(&source.name).build());
        }
        toggles.set_active_name(Some(SHOW_ALL));
    }

    /// One line per collection, with a link to its page.
    fn show_credits(&self) {
        for source in self.imp().sources.borrow().iter() {
            let link = glib::markup_escape_text(&source.link);
            let name = glib::markup_escape_text(&source.name);
            let credit = glib::markup_escape_text(&source.credit);
            let label = gtk::Label::builder()
                .label(format!("<a href=\"{link}\">{name}</a>: {credit}"))
                .use_markup(true)
                .wrap(true)
                .xalign(0.0)
                .css_classes(["dim-label", "caption"])
                .build();
            self.imp().credits_box.append(&label);
        }
    }

    /// Reads every source's live list. One failing source does not hide the others.
    fn load_wallpapers(&self) {
        let imp = self.imp();
        imp.has_loaded.set(true);
        imp.content_stack.set_visible_child_name("loading");

        glib::spawn_future_local(clone!(
            #[weak(rename_to = page)]
            self,
            async move {
                let wallpapers = page.fetch_all_sources().await;
                page.show_wallpapers(wallpapers);
            }
        ));
    }

    async fn fetch_all_sources(&self) -> Vec<Wallpaper> {
        let imp = self.imp();
        let sources = imp.sources.borrow().clone();
        let mut wallpapers = Vec::new();

        for source in &sources {
            match fetch_source(&imp.session, source).await {
                Ok(found) => wallpapers.extend(found),
                Err(error) => tracing::warn!("Could not read {}: {error:#}", source.name),
            }
        }

        wallpapers
    }

    fn show_wallpapers(&self, wallpapers: Vec<Wallpaper>) {
        let imp = self.imp();
        let allowed = remove_blocked(wallpapers, &imp.blocklist.borrow());

        if allowed.is_empty() {
            imp.has_loaded.set(false);
            imp.content_stack.set_visible_child_name("error");
            return;
        }

        imp.wallpapers.replace(allowed);
        imp.content_stack.set_visible_child_name("grid");
        self.show_first_page();
    }

    /// The wallpapers of the chosen source, or of all sources.
    fn filtered_wallpapers(&self) -> Vec<Wallpaper> {
        let imp = self.imp();
        let active = imp.source_toggles.active_name();
        let wallpapers = imp.wallpapers.borrow();

        match active.as_deref() {
            None | Some(SHOW_ALL) => wallpapers.clone(),
            Some(source_id) => wallpapers
                .iter()
                .filter(|wallpaper| wallpaper.source_id == source_id)
                .cloned()
                .collect(),
        }
    }

    fn show_first_page(&self) {
        let imp = self.imp();
        imp.wallpapers_flowbox.remove_all();
        imp.thumbnail_queue.borrow_mut().clear();
        imp.shown_count.set(0);
        self.show_next_page();
    }

    fn show_next_page(&self) {
        let imp = self.imp();
        let filtered = self.filtered_wallpapers();
        let start = imp.shown_count.get();
        let end = (start + PAGE_SIZE).min(filtered.len());

        for wallpaper in &filtered[start..end] {
            let card = WallpaperCard::new(wallpaper);
            // The card is a button, so it gets the click, not the grid.
            card.connect_clicked(clone!(
                #[weak(rename_to = page)]
                self,
                move |card| page.open_preview(card)
            ));
            imp.wallpapers_flowbox.append(&card);
            imp.thumbnail_queue.borrow_mut().push_back(card.downgrade());
        }

        imp.shown_count.set(end);
        imp.show_more_button.set_visible(end < filtered.len());
        self.load_queued_thumbnails();
    }

    /// Starts queued thumbnail downloads until four are running.
    /// Each finished download starts the next one.
    fn load_queued_thumbnails(&self) {
        let imp = self.imp();

        while imp.thumbnails_loading.get() < PARALLEL_THUMBNAILS {
            let Some(card) = self.next_queued_card() else {
                return;
            };
            imp.thumbnails_loading.set(imp.thumbnails_loading.get() + 1);

            glib::spawn_future_local(clone!(
                #[weak(rename_to = page)]
                self,
                async move {
                    card.load_thumbnail(&page.imp().session).await;
                    let imp = page.imp();
                    imp.thumbnails_loading.set(imp.thumbnails_loading.get() - 1);
                    page.load_queued_thumbnails();
                }
            ));
        }
    }

    /// The next card still in the grid. Cards removed by a filter change are skipped.
    fn next_queued_card(&self) -> Option<WallpaperCard> {
        let mut queue = self.imp().thumbnail_queue.borrow_mut();
        while let Some(weak_card) = queue.pop_front() {
            if let Some(card) = weak_card.upgrade() {
                return Some(card);
            }
        }
        None
    }

    fn open_preview(&self, card: &WallpaperCard) {
        let Some(wallpaper) = card.wallpaper() else {
            return;
        };
        let imp = self.imp();
        let source = imp
            .sources
            .borrow()
            .iter()
            .find(|source| source.id == wallpaper.source_id)
            .cloned();

        let context = WallpaperDialogContext {
            session: imp.session.clone(),
            credit: source.as_ref().map(|source| source.credit.clone()).unwrap_or_default(),
            source_link: source.map(|source| source.link).unwrap_or_default(),
            hint: imp.portal_hint.borrow().clone(),
        };

        WallpaperDialog::new(&wallpaper, card.thumbnail().as_ref(), context).present(Some(self));
    }
}

fn remove_blocked(wallpapers: Vec<Wallpaper>, blocklist: &[String]) -> Vec<Wallpaper> {
    wallpapers
        .into_iter()
        .filter(|wallpaper| !blocklist.contains(&wallpaper.full_url))
        .collect()
}

/// When the user's compositor is known and its portal backend has no Wallpaper
/// portal (Hyprland, sway and others), Save is the way to go.
fn portal_hint(catalog: &Catalog, system: &SystemInfo) -> Option<String> {
    let compositor = system.current_compositor(catalog)?;
    if compositor.has_portal(WALLPAPER_PORTAL) {
        return None;
    }

    Some(
        gettext("{} usually cannot set wallpapers from apps. Save the image, then set it with your wallpaper tool, like hyprpaper, swaybg or awww.")
            .replace("{}", &compositor.name),
    )
}
