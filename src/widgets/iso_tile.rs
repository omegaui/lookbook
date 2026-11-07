//! A single isometric slab with an icon on top: the art on protocol cards.

use std::cell::RefCell;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene, gsk};

use super::iso::{apply_floor_transform, screen_drop_in_floor};
use super::iso_stack::darken;

const SIZE: i32 = 112;
const SLAB_SIZE: f32 = 78.0;
const SLAB_RADIUS: f32 = 16.0;
const SLAB_THICKNESS: f32 = 12.0;
const ICON_SCALE: f32 = 0.5;
const SHADOW: gdk::RGBA = gdk::RGBA::new(0.0, 0.0, 0.02, 0.18);

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct IsoTile {
        pub color: RefCell<Option<gdk::RGBA>>,
        pub icon_name: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IsoTile {
        const NAME: &'static str = "IsoTile";
        type Type = super::IsoTile;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("isotile");
            klass.set_accessible_role(gtk::AccessibleRole::Presentation);
        }
    }

    impl ObjectImpl for IsoTile {}

    impl WidgetImpl for IsoTile {
        fn measure(&self, _orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            (SIZE, SIZE, -1, -1)
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let Some(color) = *self.color.borrow() else {
                return;
            };
            let widget = self.obj();
            let center = graphene::Point::new(widget.width() as f32 / 2.0, widget.height() as f32 * 0.46);

            snapshot.save();
            snapshot.translate(&center);
            apply_floor_transform(snapshot);
            draw_slab(snapshot, &color);
            if let Some(icon_name) = self.icon_name.borrow().as_deref() {
                draw_icon(snapshot, &widget, icon_name);
            }
            snapshot.restore();
        }
    }
}

glib::wrapper! {
    pub struct IsoTile(ObjectSubclass<imp::IsoTile>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl IsoTile {
    /// `color` is a CSS color such as "#1c71d8".
    pub fn set_art(&self, color: &str, icon_name: &str) {
        let imp = self.imp();
        imp.color.replace(gdk::RGBA::parse(color).ok());
        imp.icon_name.replace(Some(icon_name.to_owned()));
        self.queue_draw();
    }
}

/// A soft shadow, a darker edge, then the colored top.
fn draw_slab(snapshot: &gtk::Snapshot, color: &gdk::RGBA) {
    let half = SLAB_SIZE / 2.0;
    let top = graphene::Rect::new(-half, -half, SLAB_SIZE, SLAB_SIZE);
    let (drop_x, drop_y) = screen_drop_in_floor(SLAB_THICKNESS);
    let edge = graphene::Rect::new(-half + drop_x, -half + drop_y, SLAB_SIZE, SLAB_SIZE);
    let (shadow_x, shadow_y) = screen_drop_in_floor(SLAB_THICKNESS * 2.0);
    let shadow = graphene::Rect::new(-half + shadow_x, -half + shadow_y, SLAB_SIZE, SLAB_SIZE);

    fill_rounded(snapshot, shadow, &SHADOW);
    fill_rounded(snapshot, edge, &darken(*color, 0.35));
    fill_rounded(snapshot, top, color);
}

fn draw_icon(snapshot: &gtk::Snapshot, widget: &IsoTile, icon_name: &str) {
    let icon_theme = gtk::IconTheme::for_display(&widget.display());
    if !icon_theme.has_icon(icon_name) {
        return;
    }

    let size = SLAB_SIZE * ICON_SCALE;
    let paintable = icon_theme.lookup_icon(
        icon_name,
        &[],
        size.round() as i32,
        widget.scale_factor(),
        gtk::TextDirection::None,
        gtk::IconLookupFlags::FORCE_SYMBOLIC,
    );

    snapshot.save();
    snapshot.translate(&graphene::Point::new(-size / 2.0, -size / 2.0));
    paintable.snapshot_symbolic(snapshot, f64::from(size), f64::from(size), &[gdk::RGBA::WHITE]);
    snapshot.restore();
}

fn fill_rounded(snapshot: &gtk::Snapshot, rect: graphene::Rect, color: &gdk::RGBA) {
    snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(rect, SLAB_RADIUS));
    snapshot.append_color(color, &rect);
    snapshot.pop();
}
