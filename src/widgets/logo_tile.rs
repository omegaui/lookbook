//! A rounded tile in the item's brand color with its logo in white.
//! Items without a bundled logo show their short name instead.

use std::cell::{Cell, RefCell};

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene, gsk, pango};

const DEFAULT_SIZE: i32 = 64;
const FALLBACK_COLOR: gdk::RGBA = gdk::RGBA::new(0.37, 0.36, 0.39, 1.0);
const SHADOW_COLOR: gdk::RGBA = gdk::RGBA::new(0.0, 0.0, 0.02, 0.2);
/// How much of the tile the logo covers.
const LOGO_SCALE: f32 = 0.56;

/// The icon name of a bundled logo, for example "fedora" becomes
/// "lay-logo-fedora-symbolic". The files are in `data/resources/icons/scalable/apps/`.
pub fn logo_icon_name(logo: &str) -> String {
    format!("lay-logo-{logo}-symbolic")
}

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct LogoTile {
        pub text: RefCell<String>,
        pub logo_icon: RefCell<Option<String>>,
        pub color: Cell<gdk::RGBA>,
        pub size: Cell<i32>,
    }

    impl Default for LogoTile {
        fn default() -> Self {
            Self {
                text: RefCell::default(),
                logo_icon: RefCell::default(),
                color: Cell::new(FALLBACK_COLOR),
                size: Cell::new(DEFAULT_SIZE),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LogoTile {
        const NAME: &'static str = "LogoTile";
        type Type = super::LogoTile;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("logotile");
            klass.set_accessible_role(gtk::AccessibleRole::Presentation);
        }
    }

    impl ObjectImpl for LogoTile {}

    impl WidgetImpl for LogoTile {
        fn measure(&self, _orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            let size = self.size.get();
            (size, size, -1, -1)
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            self.draw_tile(snapshot);

            let drew_logo = self.draw_logo(snapshot);
            if !drew_logo {
                self.draw_text(snapshot);
            }
        }
    }

    impl LogoTile {
        fn draw_tile(&self, snapshot: &gtk::Snapshot) {
            let size = self.size.get() as f32;
            let bounds = graphene::Rect::new(0.0, 0.0, size, size);
            let tile = gsk::RoundedRect::from_rect(bounds, size * 0.28);

            // A solid shadow below the tile gives it a raised, game-like edge.
            snapshot.append_outset_shadow(&tile, &SHADOW_COLOR, 0.0, size * 0.06, 0.0, 0.0);

            snapshot.push_rounded_clip(&tile);
            snapshot.append_color(&self.color.get(), &bounds);
            snapshot.pop();
        }

        /// Returns false when there is no logo to draw.
        fn draw_logo(&self, snapshot: &gtk::Snapshot) -> bool {
            let logo_icon = self.logo_icon.borrow();
            let Some(icon_name) = logo_icon.as_deref() else {
                return false;
            };

            let widget = self.obj();
            let icon_theme = gtk::IconTheme::for_display(&widget.display());
            if !icon_theme.has_icon(icon_name) {
                return false;
            }

            let size = self.size.get();
            let logo_size = (size as f32 * LOGO_SCALE).round() as i32;
            let paintable = icon_theme.lookup_icon(
                icon_name,
                &[],
                logo_size,
                widget.scale_factor(),
                gtk::TextDirection::None,
                gtk::IconLookupFlags::FORCE_SYMBOLIC,
            );

            let offset = (size - logo_size) as f32 / 2.0;
            snapshot.save();
            snapshot.translate(&graphene::Point::new(offset, offset));
            paintable.snapshot_symbolic(
                snapshot,
                f64::from(logo_size),
                f64::from(logo_size),
                &[gdk::RGBA::WHITE],
            );
            snapshot.restore();

            true
        }

        fn draw_text(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let size = self.size.get();
            let layout = widget.create_pango_layout(Some(&self.text.borrow()));

            let mut font = widget.pango_context().font_description().unwrap_or_default();
            font.set_weight(pango::Weight::Heavy);
            font.set_absolute_size(f64::from(size) * 0.34 * f64::from(pango::SCALE));
            layout.set_font_description(Some(&font));

            let (_, extents) = layout.pixel_extents();
            let x = (size - extents.width()) as f32 / 2.0;
            let y = (size - extents.height()) as f32 / 2.0;

            snapshot.save();
            snapshot.translate(&graphene::Point::new(x, y));
            snapshot.append_layout(&layout, &gdk::RGBA::WHITE);
            snapshot.restore();
        }
    }
}

glib::wrapper! {
    pub struct LogoTile(ObjectSubclass<imp::LogoTile>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for LogoTile {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl LogoTile {
    pub fn new(content: &TileContent, size: i32) -> Self {
        let tile = Self::default();
        tile.set_tile_size(size);
        tile.set_content(content);
        tile
    }

    pub fn set_content(&self, content: &TileContent) {
        let imp = self.imp();
        imp.text.replace(content.short.clone());
        imp.logo_icon.replace(content.logo.as_deref().map(logo_icon_name));
        imp.color
            .set(gdk::RGBA::parse(content.color.as_str()).unwrap_or(FALLBACK_COLOR));
        self.queue_draw();
    }

    pub fn set_tile_size(&self, size: i32) {
        self.imp().size.set(size);
        self.queue_resize();
    }
}

/// What a tile shows: a logo when there is one, otherwise the short name, on `color`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileContent {
    pub short: String,
    pub color: String,
    /// A bundled logo id, for example "fedora".
    pub logo: Option<String>,
}
