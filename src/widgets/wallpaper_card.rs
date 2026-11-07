use std::cell::RefCell;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib};

use crate::wallpapers::{fetch_texture, Wallpaper};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/LinuxAndYou/ui/widgets/wallpaper_card.ui")]
    pub struct WallpaperCard {
        #[template_child]
        pub picture: gtk::TemplateChild<gtk::Picture>,
        #[template_child]
        pub title_label: gtk::TemplateChild<gtk::Label>,
        pub wallpaper: RefCell<Option<Wallpaper>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WallpaperCard {
        const NAME: &'static str = "WallpaperCard";
        type Type = super::WallpaperCard;
        type ParentType = gtk::Button;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for WallpaperCard {}
    impl WidgetImpl for WallpaperCard {}
    impl ButtonImpl for WallpaperCard {}
}

glib::wrapper! {
    /// A wallpaper thumbnail in the Wallpapers grid. Clicking it opens the preview.
    pub struct WallpaperCard(ObjectSubclass<imp::WallpaperCard>)
        @extends gtk::Widget, gtk::Button,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl WallpaperCard {
    pub fn new(wallpaper: &Wallpaper) -> Self {
        let card: Self = glib::Object::new();
        let imp = card.imp();

        imp.title_label.set_label(&wallpaper.title);
        imp.picture
            .update_property(&[gtk::accessible::Property::Label(&wallpaper.title)]);
        imp.wallpaper.replace(Some(wallpaper.clone()));
        card
    }

    pub fn wallpaper(&self) -> Option<Wallpaper> {
        self.imp().wallpaper.borrow().clone()
    }

    /// The thumbnail once it has loaded, to show while the full image downloads.
    pub fn thumbnail(&self) -> Option<gdk::Paintable> {
        self.imp().picture.paintable()
    }

    /// Downloads the thumbnail from the wallpaper's source and shows it.
    pub async fn load_thumbnail(&self, session: &soup::Session) {
        let Some(wallpaper) = self.wallpaper() else {
            return;
        };

        match fetch_texture(session, &wallpaper.thumbnail_url).await {
            Ok(texture) => self.imp().picture.set_paintable(Some(&texture)),
            Err(error) => tracing::warn!("Could not load the thumbnail of {}: {error:#}", wallpaper.title),
        }
    }
}
