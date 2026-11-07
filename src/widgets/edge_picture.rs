//! A screenshot or wallpaper shown straight on, running off the right and
//! bottom edges of its banner. Until the image arrives it shows a soft gradient.

use std::cell::RefCell;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene, gsk};

use super::edge::{drop_shadow, screen_rect, SCREEN_RADIUS};

const PLACEHOLDER_TOP: gdk::RGBA = gdk::RGBA::new(0.6, 0.6, 0.65, 0.5);
const PLACEHOLDER_BOTTOM: gdk::RGBA = gdk::RGBA::new(0.4, 0.4, 0.45, 0.5);

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct EdgePicture {
        pub texture: RefCell<Option<gdk::Texture>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EdgePicture {
        const NAME: &'static str = "EdgePicture";
        type Type = super::EdgePicture;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("edgepicture");
            klass.set_accessible_role(gtk::AccessibleRole::Img);
        }
    }

    impl ObjectImpl for EdgePicture {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_overflow(gtk::Overflow::Hidden);
        }
    }

    impl WidgetImpl for EdgePicture {
        /// It can shrink to nothing; the picture is cropped, never squeezed.
        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                gtk::Orientation::Horizontal => (0, 480, -1, -1),
                _ => (300, 340, -1, -1),
            }
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let screen = screen_rect(self.obj().width() as f32);
            drop_shadow(snapshot, &screen, SCREEN_RADIUS, 8.0);

            snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(screen, SCREEN_RADIUS));
            match self.texture.borrow().as_ref() {
                Some(texture) => draw_cover(snapshot, texture, &screen),
                None => draw_placeholder(snapshot, &screen),
            }
            snapshot.pop();
        }
    }
}

glib::wrapper! {
    pub struct EdgePicture(ObjectSubclass<imp::EdgePicture>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for EdgePicture {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl EdgePicture {
    pub fn set_texture(&self, texture: Option<&gdk::Texture>) {
        self.imp().texture.replace(texture.cloned());
        self.queue_draw();
    }
}

/// Scales the image to cover the whole screen, cropping what spills over.
fn draw_cover(snapshot: &gtk::Snapshot, texture: &gdk::Texture, screen: &graphene::Rect) {
    let image_width = texture.width() as f32;
    let image_height = texture.height() as f32;
    if image_width <= 0.0 || image_height <= 0.0 {
        return;
    }

    let scale = (screen.width() / image_width).max(screen.height() / image_height);
    let width = image_width * scale;
    let height = image_height * scale;
    let x = screen.x() + (screen.width() - width) / 2.0;
    let y = screen.y() + (screen.height() - height) / 2.0;
    let area = graphene::Rect::new(x, y, width, height);
    snapshot.append_scaled_texture(texture, gsk::ScalingFilter::Trilinear, &area);
}

fn draw_placeholder(snapshot: &gtk::Snapshot, screen: &graphene::Rect) {
    let start = graphene::Point::new(screen.x(), screen.y());
    let end = graphene::Point::new(screen.x(), screen.y() + screen.height());
    snapshot.append_linear_gradient(
        screen,
        &start,
        &end,
        &[
            gsk::ColorStop::new(0.0, PLACEHOLDER_TOP),
            gsk::ColorStop::new(1.0, PLACEHOLDER_BOTTOM),
        ],
    );
}
