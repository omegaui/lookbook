//! A wrapper that reports a smaller natural width than its child wants, while
//! still letting the child fill whatever width it is given.
//!
//! A picture's natural width is its image's full width. In a flow box that
//! leaves room for only one card per row; capping the natural width lets cards
//! share a row and still stretch to fill it.
//!
//! It is a plain widget, not an `adw::Bin`: a widget with a layout manager
//! never has its own `measure` called.

use std::cell::{Cell, RefCell};

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct NaturalWidthCap {
        pub cap: Cell<i32>,
        pub child: RefCell<Option<gtk::Widget>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NaturalWidthCap {
        const NAME: &'static str = "NaturalWidthCap";
        type Type = super::NaturalWidthCap;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for NaturalWidthCap {
        fn dispose(&self) {
            if let Some(child) = self.child.take() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for NaturalWidthCap {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            let Some(child) = self.child.borrow().clone() else {
                return (0, 0, -1, -1);
            };
            let (minimum, natural, minimum_baseline, natural_baseline) =
                child.measure(orientation, for_size);
            if orientation != gtk::Orientation::Horizontal {
                return (minimum, natural, minimum_baseline, natural_baseline);
            }
            let capped = natural.min(self.cap.get()).max(minimum);
            (minimum, capped, -1, -1)
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            if let Some(child) = self.child.borrow().as_ref() {
                child.allocate(width, height, baseline, None);
            }
        }

        fn request_mode(&self) -> gtk::SizeRequestMode {
            self.child
                .borrow()
                .as_ref()
                .map_or(gtk::SizeRequestMode::ConstantSize, |child| child.request_mode())
        }
    }
}

glib::wrapper! {
    pub struct NaturalWidthCap(ObjectSubclass<imp::NaturalWidthCap>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl NaturalWidthCap {
    pub fn new(child: &impl IsA<gtk::Widget>, cap: i32) -> Self {
        let wrapper: Self = glib::Object::new();
        let imp = wrapper.imp();
        imp.cap.set(cap);
        child.set_parent(&wrapper);
        imp.child.replace(Some(child.clone().upcast()));
        wrapper
    }
}
