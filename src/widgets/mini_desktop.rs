//! A small isometric drawing of a desktop for the Desktops gallery: a wallpaper
//! slab in the desktop's color, one window, and the bars it shows by default.

use std::cell::{Cell, RefCell};

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene, gsk};

use super::iso::{apply_floor_transform, floor_to_screen, screen_drop_in_floor};
use crate::catalog::DesktopBar;

const SLAB_WIDTH: f32 = 176.0;
const SLAB_HEIGHT: f32 = 110.0;
const SLAB_RADIUS: f32 = 12.0;
const SLAB_THICKNESS: f32 = 8.0;
const AREA_HEIGHT: i32 = 150;
const AREA_WIDTH: i32 = 240;
/// How strongly the desktop's color tints the area behind the slab.
const TINT_ALPHA: f32 = 0.16;

const BAR: gdk::RGBA = gdk::RGBA::new(0.14, 0.12, 0.19, 1.0);
const WINDOW_FACE: gdk::RGBA = gdk::RGBA::new(0.98, 0.98, 0.98, 1.0);
const WINDOW_TITLE: gdk::RGBA = gdk::RGBA::new(0.92, 0.92, 0.93, 1.0);
const WINDOW_EDGE: gdk::RGBA = gdk::RGBA::new(0.69, 0.69, 0.71, 1.0);
const DOCK: gdk::RGBA = gdk::RGBA::new(1.0, 1.0, 1.0, 0.75);

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct MiniDesktop {
        pub color: Cell<Option<gdk::RGBA>>,
        pub bars: RefCell<Vec<DesktopBar>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MiniDesktop {
        const NAME: &'static str = "MiniDesktop";
        type Type = super::MiniDesktop;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("minidesktop");
            klass.set_accessible_role(gtk::AccessibleRole::Presentation);
        }
    }

    impl ObjectImpl for MiniDesktop {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_overflow(gtk::Overflow::Hidden);
        }
    }

    impl WidgetImpl for MiniDesktop {
        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                gtk::Orientation::Horizontal => (0, AREA_WIDTH, -1, -1),
                _ => (AREA_HEIGHT, AREA_HEIGHT, -1, -1),
            }
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let Some(color) = self.color.get() else {
                return;
            };
            let widget = self.obj();
            let width = widget.width() as f32;
            let height = widget.height() as f32;

            draw_tint(snapshot, color, width, height);

            snapshot.save();
            snapshot.translate(&slab_origin(width, height));
            apply_floor_transform(snapshot);
            draw_slab(snapshot, color);
            draw_window(snapshot);
            draw_bars(snapshot, &self.bars.borrow());
            snapshot.restore();
        }
    }
}

glib::wrapper! {
    pub struct MiniDesktop(ObjectSubclass<imp::MiniDesktop>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for MiniDesktop {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl MiniDesktop {
    /// Draws the desktop in `color`, a brand color such as "#1c71d8".
    pub fn set_desktop(&self, color: &str, bars: &[DesktopBar]) {
        let imp = self.imp();
        imp.color.set(gdk::RGBA::parse(color).ok());
        imp.bars.replace(bars.to_vec());
        self.queue_draw();
    }
}

/// A soft wash of the desktop's color behind the slab, readable in light and dark styles.
fn draw_tint(snapshot: &gtk::Snapshot, color: gdk::RGBA, width: f32, height: f32) {
    let tint = gdk::RGBA::new(color.red(), color.green(), color.blue(), TINT_ALPHA);
    snapshot.append_color(&tint, &graphene::Rect::new(0.0, 0.0, width, height));
}

/// Where the slab's top-left corner goes so the tilted slab sits in the middle.
fn slab_origin(width: f32, height: f32) -> graphene::Point {
    let corners = [
        floor_to_screen(0.0, 0.0),
        floor_to_screen(SLAB_WIDTH, 0.0),
        floor_to_screen(0.0, SLAB_HEIGHT),
        floor_to_screen(SLAB_WIDTH, SLAB_HEIGHT),
    ];
    let left = corners.iter().map(|point| point.x()).fold(f32::MAX, f32::min);
    let right = corners.iter().map(|point| point.x()).fold(f32::MIN, f32::max);
    let top = corners.iter().map(|point| point.y()).fold(f32::MAX, f32::min);
    let bottom = corners.iter().map(|point| point.y()).fold(f32::MIN, f32::max);

    let x = (width - (right - left)) / 2.0 - left;
    let y = (height - (bottom - top)) / 2.0 - top - SLAB_THICKNESS / 2.0;
    graphene::Point::new(x, y)
}

fn draw_slab(snapshot: &gtk::Snapshot, color: gdk::RGBA) {
    let slab = graphene::Rect::new(0.0, 0.0, SLAB_WIDTH, SLAB_HEIGHT);
    let (dx, dy) = screen_drop_in_floor(SLAB_THICKNESS);
    fill_rounded(snapshot, offset(slab, dx, dy), SLAB_RADIUS, &shade(color, 0.0, 0.35));
    fill_rounded(snapshot, slab, SLAB_RADIUS, &shade(color, 0.25, 0.0));
}

fn draw_window(snapshot: &gtk::Snapshot) {
    let window = graphene::Rect::new(28.0, 24.0, 92.0, 58.0);
    let (dx, dy) = screen_drop_in_floor(4.0);
    fill_rounded(snapshot, offset(window, dx, dy), 7.0, &WINDOW_EDGE);
    fill_rounded(snapshot, window, 7.0, &WINDOW_FACE);

    let title = graphene::Rect::new(window.x(), window.y(), window.width(), 12.0);
    snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(window, 7.0));
    snapshot.append_color(&WINDOW_TITLE, &title);
    snapshot.pop();
}

fn draw_bars(snapshot: &gtk::Snapshot, bars: &[DesktopBar]) {
    let slab = graphene::Rect::new(0.0, 0.0, SLAB_WIDTH, SLAB_HEIGHT);
    snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(slab, SLAB_RADIUS));
    for bar in bars {
        match bar {
            DesktopBar::Top => {
                snapshot.append_color(&BAR, &graphene::Rect::new(0.0, 0.0, SLAB_WIDTH, 12.0));
            }
            DesktopBar::Bottom => {
                let top = SLAB_HEIGHT - 16.0;
                snapshot.append_color(&BAR, &graphene::Rect::new(0.0, top, SLAB_WIDTH, 16.0));
            }
            DesktopBar::Dock => {
                let dock = graphene::Rect::new((SLAB_WIDTH - 70.0) / 2.0, SLAB_HEIGHT - 21.0, 70.0, 13.0);
                fill_rounded(snapshot, dock, 6.0, &DOCK);
            }
        }
    }
    snapshot.pop();
}

/// Mixes `color` toward white by `lighten` and toward black by `darken`.
fn shade(color: gdk::RGBA, lighten: f32, darken: f32) -> gdk::RGBA {
    let channel = |value: f32| {
        let lighter = value + (1.0 - value) * lighten;
        lighter * (1.0 - darken)
    };
    gdk::RGBA::new(channel(color.red()), channel(color.green()), channel(color.blue()), 1.0)
}

fn fill_rounded(snapshot: &gtk::Snapshot, rect: graphene::Rect, radius: f32, color: &gdk::RGBA) {
    snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(rect, radius));
    snapshot.append_color(color, &rect);
    snapshot.pop();
}

fn offset(rect: graphene::Rect, dx: f32, dy: f32) -> graphene::Rect {
    graphene::Rect::new(rect.x() + dx, rect.y() + dy, rect.width(), rect.height())
}
