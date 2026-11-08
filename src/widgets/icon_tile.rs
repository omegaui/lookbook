//! A rounded square in a color with a white icon on it: the art on protocol cards.

use std::cell::RefCell;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene, gsk};

use super::edge::drop_shadow;

const SIZE: i32 = 76;
const TILE_SIZE: f32 = 56.0;
const TILE_RADIUS: f32 = 16.0;
const ICON_SCALE: f32 = 0.5;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct IconTile {
        pub color: RefCell<Option<gdk::RGBA>>,
        pub icon_name: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IconTile {
        const NAME: &'static str = "IconTile";
        type Type = super::IconTile;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("icontile");
            klass.set_accessible_role(gtk::AccessibleRole::Presentation);
        }
    }

    impl ObjectImpl for IconTile {}

    impl WidgetImpl for IconTile {
        fn measure(&self, _orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            (SIZE, SIZE, -1, -1)
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let Some(color) = *self.color.borrow() else {
                return;
            };
            let widget = self.obj();
            let tile = graphene::Rect::new(
                (widget.width() as f32 - TILE_SIZE) / 2.0,
                (widget.height() as f32 - TILE_SIZE) / 2.0,
                TILE_SIZE,
                TILE_SIZE,
            );

            drop_shadow(snapshot, &tile, TILE_RADIUS, 3.0);
            snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(tile, TILE_RADIUS));
            snapshot.append_color(&color, &tile);
            snapshot.pop();

            if let Some(icon_name) = self.icon_name.borrow().as_deref() {
                draw_icon(snapshot, &widget, icon_name, &tile);
            }
        }
    }
}

glib::wrapper! {
    pub struct IconTile(ObjectSubclass<imp::IconTile>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl IconTile {
    /// `color` is a CSS color such as "#1c71d8".
    pub fn set_art(&self, color: &str, icon_name: &str) {
        let imp = self.imp();
        imp.color.replace(gdk::RGBA::parse(color).ok());
        imp.icon_name.replace(Some(icon_name.to_owned()));
        self.queue_draw();
    }
}

/// A white symbolic icon in the middle of the tile.
fn draw_icon(snapshot: &gtk::Snapshot, widget: &IconTile, icon_name: &str, tile: &graphene::Rect) {
    let icon_theme = gtk::IconTheme::for_display(&widget.display());
    if !icon_theme.has_icon(icon_name) {
        return;
    }

    let size = TILE_SIZE * ICON_SCALE;
    let paintable = icon_theme.lookup_icon(
        icon_name,
        &[],
        size.round() as i32,
        widget.scale_factor(),
        gtk::TextDirection::None,
        gtk::IconLookupFlags::FORCE_SYMBOLIC,
    );

    snapshot.save();
    snapshot.translate(&graphene::Point::new(
        tile.x() + (TILE_SIZE - size) / 2.0,
        tile.y() + (TILE_SIZE - size) / 2.0,
    ));
    paintable.snapshot_symbolic(snapshot, f64::from(size), f64::from(size), &[gdk::RGBA::WHITE]);
    snapshot.restore();
}
