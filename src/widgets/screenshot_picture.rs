//! A screenshot in a 16:10 frame with rounded corners. It shows a soft
//! placeholder until the image arrives, then covers the frame with it, or,
//! for galleries, shows the whole image in the middle of the frame.

use std::cell::{Cell, RefCell};

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene, gsk};

const ASPECT_WIDTH: i32 = 16;
const ASPECT_HEIGHT: i32 = 10;
const CORNER_RADIUS: f32 = 14.0;
const PLACEHOLDER: gdk::RGBA = gdk::RGBA::new(0.5, 0.5, 0.55, 0.15);

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct ScreenshotPicture {
        pub(super) texture: RefCell<Option<gdk::Texture>>,
        /// Show the whole image instead of cropping it to fill the frame.
        pub(super) show_whole: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScreenshotPicture {
        const NAME: &'static str = "ScreenshotPicture";
        type Type = super::ScreenshotPicture;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("screenshotpicture");
            klass.set_accessible_role(gtk::AccessibleRole::Img);
        }
    }

    impl ObjectImpl for ScreenshotPicture {}

    impl WidgetImpl for ScreenshotPicture {
        fn request_mode(&self) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::HeightForWidth
        }

        /// Keeps a 16:10 shape at any width.
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            if orientation == gtk::Orientation::Horizontal {
                return (64, 320, -1, -1);
            }
            let width = if for_size > 0 { for_size } else { 320 };
            let height = width * ASPECT_HEIGHT / ASPECT_WIDTH;
            (height, height, -1, -1)
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let bounds = graphene::Rect::new(0.0, 0.0, widget.width() as f32, widget.height() as f32);
            match self.texture.borrow().as_ref() {
                Some(texture) if self.show_whole.get() => draw_whole(snapshot, texture, &bounds),
                Some(texture) => draw_cover(snapshot, texture, &bounds),
                None => fill_rounded(snapshot, &bounds, &PLACEHOLDER),
            }
        }
    }
}

glib::wrapper! {
    pub struct ScreenshotPicture(ObjectSubclass<imp::ScreenshotPicture>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for ScreenshotPicture {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl ScreenshotPicture {
    pub fn set_texture(&self, texture: &gdk::Texture) {
        self.imp().texture.replace(Some(texture.clone()));
        self.queue_draw();
    }

    /// Galleries show each screenshot whole, whatever its shape.
    pub fn set_show_whole(&self, show_whole: bool) {
        self.imp().show_whole.set(show_whole);
        self.queue_draw();
    }
}

/// Scales the screenshot to cover the whole frame, cropping what spills over.
fn draw_cover(snapshot: &gtk::Snapshot, texture: &gdk::Texture, bounds: &graphene::Rect) {
    let image_width = texture.width() as f32;
    let image_height = texture.height() as f32;
    if image_width <= 0.0 || image_height <= 0.0 {
        return;
    }

    snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(*bounds, CORNER_RADIUS));
    let scale = (bounds.width() / image_width).max(bounds.height() / image_height);
    let drawn_width = image_width * scale;
    let drawn_height = image_height * scale;
    let area = graphene::Rect::new(
        (bounds.width() - drawn_width) / 2.0,
        (bounds.height() - drawn_height) / 2.0,
        drawn_width,
        drawn_height,
    );
    snapshot.append_scaled_texture(texture, gsk::ScalingFilter::Trilinear, &area);
    snapshot.pop();
}

/// Scales the screenshot to fit inside the frame, centred, with its own
/// rounded corners.
fn draw_whole(snapshot: &gtk::Snapshot, texture: &gdk::Texture, bounds: &graphene::Rect) {
    let image_width = texture.width() as f32;
    let image_height = texture.height() as f32;
    if image_width <= 0.0 || image_height <= 0.0 {
        return;
    }

    let scale = (bounds.width() / image_width).min(bounds.height() / image_height);
    let width = image_width * scale;
    let height = image_height * scale;
    let area = graphene::Rect::new(
        (bounds.width() - width) / 2.0,
        (bounds.height() - height) / 2.0,
        width,
        height,
    );
    snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(area, CORNER_RADIUS));
    snapshot.append_scaled_texture(texture, gsk::ScalingFilter::Trilinear, &area);
    snapshot.pop();
}

fn fill_rounded(snapshot: &gtk::Snapshot, rect: &graphene::Rect, color: &gdk::RGBA) {
    snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(*rect, CORNER_RADIUS));
    snapshot.append_color(color, rect);
    snapshot.pop();
}
