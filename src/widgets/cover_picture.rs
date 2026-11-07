//! A picture of a fixed height that fills any width, cropping the image to
//! cover it. Unlike `gtk::Picture`, its size never depends on the image, so
//! cards in a row stay the same height whatever shape their screenshots are.

use std::cell::{Cell, RefCell};

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene};

/// The width the picture asks for, so three cards share a row on a wide window.
const NATURAL_WIDTH: i32 = 300;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct CoverPicture {
        pub texture: RefCell<Option<gdk::Texture>>,
        pub height: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CoverPicture {
        const NAME: &'static str = "CoverPicture";
        type Type = super::CoverPicture;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("coverpicture");
            klass.set_accessible_role(gtk::AccessibleRole::Img);
        }
    }

    impl ObjectImpl for CoverPicture {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_overflow(gtk::Overflow::Hidden);
        }
    }

    impl WidgetImpl for CoverPicture {
        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                gtk::Orientation::Horizontal => (0, NATURAL_WIDTH, -1, -1),
                _ => (self.height.get(), self.height.get(), -1, -1),
            }
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let Some(texture) = self.texture.borrow().clone() else {
                return;
            };
            let widget = self.obj();
            draw_cover(snapshot, &texture, widget.width() as f32, widget.height() as f32);
        }
    }
}

glib::wrapper! {
    pub struct CoverPicture(ObjectSubclass<imp::CoverPicture>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl CoverPicture {
    pub fn new(texture: &gdk::Texture, height: i32) -> Self {
        let picture: Self = glib::Object::new();
        let imp = picture.imp();
        imp.texture.replace(Some(texture.clone()));
        imp.height.set(height);
        picture
    }
}

/// Scales the image to cover the whole area, centred, cropping what spills over.
fn draw_cover(snapshot: &gtk::Snapshot, texture: &gdk::Texture, width: f32, height: f32) {
    let image_width = texture.width() as f32;
    let image_height = texture.height() as f32;
    if image_width <= 0.0 || image_height <= 0.0 {
        return;
    }

    let scale = (width / image_width).max(height / image_height);
    let drawn_width = image_width * scale;
    let drawn_height = image_height * scale;
    let area = graphene::Rect::new(
        (width - drawn_width) / 2.0,
        (height - drawn_height) / 2.0,
        drawn_width,
        drawn_height,
    );
    snapshot.push_clip(&graphene::Rect::new(0.0, 0.0, width, height));
    snapshot.append_texture(texture, &area);
    snapshot.pop();
}
