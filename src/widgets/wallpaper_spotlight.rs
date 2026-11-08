//! The last spotlight on Explore: a wallpaper picked at random from one source,
//! read live the first time the carousel reaches it.

use std::cell::{Cell, RefCell};

use gettextrs::gettext;
use gtk::glib;
use gtk::glib::clone;
use adw::prelude::*;
use gtk::subclass::prelude::*;

use super::{chip_label, EdgePicture, LogoTile, TileContent, WallpaperDialog, WallpaperDialogContext};
use crate::catalog::{Catalog, WallpaperSource, WallpaperSpotlight as SpotlightData};
use crate::system::SystemInfo;
use crate::http::wallpaper_session;
use crate::wallpapers::{fetch_source, fetch_texture, portal_hint, remove_blocked, Wallpaper};

const TILE_ICON: &str = "lay-wallpapers-symbolic";
const TILE_COLOR: &str = "#26a269";

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Syspedia/ui/widgets/wallpaper_spotlight.ui")]
    pub struct WallpaperSpotlight {
        #[template_child]
        pub tile: gtk::TemplateChild<LogoTile>,
        #[template_child]
        pub title_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub tagline_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub chips_box: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub view_button: gtk::TemplateChild<gtk::Button>,
        #[template_child]
        pub picture: gtk::TemplateChild<EdgePicture>,
        #[template_child]
        pub spinner: gtk::TemplateChild<adw::Spinner>,
        pub source: RefCell<Option<WallpaperSource>>,
        pub blocklist: RefCell<Vec<String>>,
        pub hint: RefCell<Option<String>>,
        pub session: RefCell<Option<soup::Session>>,
        pub wallpaper: RefCell<Option<Wallpaper>>,
        pub thumbnail: RefCell<Option<gtk::gdk::Texture>>,
        pub started: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WallpaperSpotlight {
        const NAME: &'static str = "WallpaperSpotlight";
        type Type = super::WallpaperSpotlight;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            LogoTile::ensure_type();
            EdgePicture::ensure_type();
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for WallpaperSpotlight {}
    impl WidgetImpl for WallpaperSpotlight {}
    impl BoxImpl for WallpaperSpotlight {}

    #[gtk::template_callbacks]
    impl WallpaperSpotlight {
        #[template_callback]
        fn on_view_clicked(&self) {
            self.obj().open_preview();
        }
    }
}

glib::wrapper! {
    pub struct WallpaperSpotlight(ObjectSubclass<imp::WallpaperSpotlight>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl WallpaperSpotlight {
    /// Returns nothing when the catalog does not list the spotlight's source.
    pub fn new(spotlight: &SpotlightData, catalog: &Catalog, system: &SystemInfo) -> Option<Self> {
        let source = catalog
            .wallpaper_sources
            .iter()
            .find(|source| source.id == spotlight.source_id)?;

        let card: Self = glib::Object::new();
        card.show_intro(spotlight, source);
        card.store_details(source, catalog, system);
        Some(card)
    }

    /// Reads the source and shows one of its wallpapers. Only the first call does anything.
    pub fn load(&self) {
        let imp = self.imp();
        if imp.started.replace(true) {
            return;
        }

        glib::spawn_future_local(clone!(
            #[weak(rename_to = card)]
            self,
            async move { card.pick_and_show_wallpaper().await }
        ));
    }

    fn show_intro(&self, spotlight: &SpotlightData, source: &WallpaperSource) {
        let imp = self.imp();
        self.add_css_class(spotlight.accent.css_class());
        imp.tile.set_content(&TileContent {
            short: String::new(),
            color: TILE_COLOR.to_owned(),
            logo: None,
            icon_name: Some(TILE_ICON.to_owned()),
        });
        imp.title_label.set_label(&source.name);
        imp.tagline_label.set_label(&spotlight.tagline);
        imp.chips_box.append(&chip_label(&source.name));
        imp.chips_box.append(&chip_label(&gettext("Live")));
    }

    fn store_details(&self, source: &WallpaperSource, catalog: &Catalog, system: &SystemInfo) {
        let imp = self.imp();
        imp.source.replace(Some(source.clone()));
        imp.blocklist.replace(catalog.wallpaper_blocklist.clone());
        imp.hint.replace(portal_hint(catalog, system));
    }

    async fn pick_and_show_wallpaper(&self) {
        let session = wallpaper_session();
        self.imp().session.replace(Some(session.clone()));

        let Some(wallpaper) = self.pick_wallpaper(&session).await else {
            self.show_unavailable();
            return;
        };
        self.imp().title_label.set_label(&wallpaper.title);

        match fetch_texture(&session, &wallpaper.thumbnail_url).await {
            Ok(texture) => self.show_wallpaper(wallpaper, texture),
            Err(error) => {
                tracing::warn!("Could not load the spotlight wallpaper: {error:#}");
                self.show_unavailable();
            }
        }
    }

    /// One wallpaper at random, so the spotlight changes each time the app opens.
    async fn pick_wallpaper(&self, session: &soup::Session) -> Option<Wallpaper> {
        let source = self.imp().source.borrow().clone()?;
        let wallpapers = match fetch_source(session, &source).await {
            Ok(wallpapers) => wallpapers,
            Err(error) => {
                tracing::warn!("Could not read {}: {error:#}", source.name);
                return None;
            }
        };

        let mut allowed = remove_blocked(wallpapers, &self.imp().blocklist.borrow());
        if allowed.is_empty() {
            return None;
        }
        let index = glib::random_int_range(0, allowed.len() as i32) as usize;
        Some(allowed.swap_remove(index))
    }

    fn show_wallpaper(&self, wallpaper: Wallpaper, texture: gtk::gdk::Texture) {
        let imp = self.imp();
        imp.picture.set_texture(Some(&texture));
        imp.picture
            .update_property(&[gtk::accessible::Property::Label(&wallpaper.title)]);
        imp.spinner.set_visible(false);
        imp.view_button.set_sensitive(true);
        imp.thumbnail.replace(Some(texture));
        imp.wallpaper.replace(Some(wallpaper));
    }

    fn show_unavailable(&self) {
        let imp = self.imp();
        imp.spinner.set_visible(false);
        imp.tagline_label
            .set_label(&gettext("The collection could not be reached right now. Browse Wallpapers to try again."));
    }

    fn open_preview(&self) {
        let imp = self.imp();
        let wallpaper = imp.wallpaper.borrow().clone();
        let source = imp.source.borrow().clone();
        let session = imp.session.borrow().clone();
        let (Some(wallpaper), Some(source), Some(session)) = (wallpaper, source, session) else {
            return;
        };

        let context = WallpaperDialogContext {
            session,
            credit: source.credit,
            source_link: source.link,
            hint: imp.hint.borrow().clone(),
        };
        let thumbnail = imp
            .thumbnail
            .borrow()
            .clone()
            .map(|texture| texture.upcast::<gtk::gdk::Paintable>());

        WallpaperDialog::new(&wallpaper, thumbnail.as_ref(), context).present(Some(self));
    }
}
