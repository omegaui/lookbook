//! A picture laid on the isometric floor like a thick tile, used for the
//! wallpaper spotlight. Until the image arrives, the tile shows a soft gradient.

use std::cell::RefCell;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene, gsk};

use super::iso::{apply_floor_transform, floor_to_screen, screen_drop_in_floor};

const TILE_WIDTH: f32 = 460.0;
/// Wallpapers are usually 16:9 or 16:10; 16:10 crops either one well.
const TILE_HEIGHT: f32 = 288.0;
const TILE_RADIUS: f32 = 18.0;
const TILE_THICKNESS: f32 = 14.0;

const EDGE: gdk::RGBA = gdk::RGBA::new(0.14, 0.12, 0.19, 1.0);
const PLACEHOLDER_TOP: gdk::RGBA = gdk::RGBA::new(0.6, 0.6, 0.65, 0.5);
const PLACEHOLDER_BOTTOM: gdk::RGBA = gdk::RGBA::new(0.4, 0.4, 0.45, 0.5);

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct IsoPicture {
        pub texture: RefCell<Option<gdk::Texture>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IsoPicture {
        const NAME: &'static str = "IsoPicture";
        type Type = super::IsoPicture;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("isopicture");
            klass.set_accessible_role(gtk::AccessibleRole::Img);
        }
    }

    impl ObjectImpl for IsoPicture {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_overflow(gtk::Overflow::Hidden);
        }
    }

    impl WidgetImpl for IsoPicture {
        /// It can shrink to nothing; the tile is cropped, never squeezed.
        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                gtk::Orientation::Horizontal => (0, 480, -1, -1),
                _ => (300, 340, -1, -1),
            }
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let width = widget.width() as f32;
            let height = widget.height() as f32;

            snapshot.save();
            snapshot.translate(&tile_origin(width, height));
            apply_floor_transform(snapshot);
            draw_tile(snapshot, self.texture.borrow().as_ref());
            snapshot.restore();
        }
    }
}

glib::wrapper! {
    pub struct IsoPicture(ObjectSubclass<imp::IsoPicture>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for IsoPicture {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl IsoPicture {
    pub fn set_texture(&self, texture: Option<&gdk::Texture>) {
        self.imp().texture.replace(texture.cloned());
        self.queue_draw();
    }
}

/// Where the tile's top-left corner goes so the tilted tile sits in the middle.
fn tile_origin(width: f32, height: f32) -> graphene::Point {
    let corners = [
        floor_to_screen(0.0, 0.0),
        floor_to_screen(TILE_WIDTH, 0.0),
        floor_to_screen(0.0, TILE_HEIGHT),
        floor_to_screen(TILE_WIDTH, TILE_HEIGHT),
    ];
    let left = corners.iter().map(|point| point.x()).fold(f32::MAX, f32::min);
    let right = corners.iter().map(|point| point.x()).fold(f32::MIN, f32::max);
    let top = corners.iter().map(|point| point.y()).fold(f32::MAX, f32::min);
    let bottom = corners.iter().map(|point| point.y()).fold(f32::MIN, f32::max);

    let x = (width - (right - left)) / 2.0 - left;
    let y = (height - (bottom - top)) / 2.0 - top - TILE_THICKNESS / 2.0;
    graphene::Point::new(x, y)
}

fn draw_tile(snapshot: &gtk::Snapshot, texture: Option<&gdk::Texture>) {
    let tile = graphene::Rect::new(0.0, 0.0, TILE_WIDTH, TILE_HEIGHT);
    let (dx, dy) = screen_drop_in_floor(TILE_THICKNESS);
    let edge = graphene::Rect::new(dx, dy, TILE_WIDTH, TILE_HEIGHT);

    snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(edge, TILE_RADIUS));
    snapshot.append_color(&EDGE, &edge);
    snapshot.pop();

    snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(tile, TILE_RADIUS));
    match texture {
        Some(texture) => draw_cover(snapshot, texture, tile),
        None => draw_placeholder(snapshot, tile),
    }
    snapshot.pop();
}

/// Scales the image to cover the whole tile, cropping what spills over.
fn draw_cover(snapshot: &gtk::Snapshot, texture: &gdk::Texture, tile: graphene::Rect) {
    let image_width = texture.width() as f32;
    let image_height = texture.height() as f32;
    if image_width <= 0.0 || image_height <= 0.0 {
        return;
    }

    let scale = (tile.width() / image_width).max(tile.height() / image_height);
    let width = image_width * scale;
    let height = image_height * scale;
    let x = tile.x() + (tile.width() - width) / 2.0;
    let y = tile.y() + (tile.height() - height) / 2.0;
    snapshot.append_texture(texture, &graphene::Rect::new(x, y, width, height));
}

fn draw_placeholder(snapshot: &gtk::Snapshot, tile: graphene::Rect) {
    let start = graphene::Point::new(tile.x(), tile.y());
    let end = graphene::Point::new(tile.x(), tile.y() + tile.height());
    snapshot.append_linear_gradient(
        &tile,
        &start,
        &end,
        &[
            gsk::ColorStop::new(0.0, PLACEHOLDER_TOP),
            gsk::ColorStop::new(1.0, PLACEHOLDER_BOTTOM),
        ],
    );
}
