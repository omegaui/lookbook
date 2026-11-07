//! Draws a stand-in desktop screenshot from a few colors.
//! It is replaced by real screenshots once the catalog has them.

use std::cell::Cell;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene, gsk};

use crate::catalog::{MockLayout, MockLook};

const ASPECT_WIDTH: i32 = 16;
const ASPECT_HEIGHT: i32 = 10;
const CORNER_RADIUS: f32 = 14.0;
const WINDOW_RADIUS: f32 = 10.0;
const WINDOW_SHADOW: gdk::RGBA = gdk::RGBA::new(0.0, 0.0, 0.02, 0.35);
const DOCK_COLOR: gdk::RGBA = gdk::RGBA::new(1.0, 1.0, 1.0, 0.35);

#[derive(Debug, Clone, Copy)]
struct Palette {
    layout: MockLayout,
    wallpaper: gdk::RGBA,
    glow: gdk::RGBA,
    bar: gdk::RGBA,
    window: gdk::RGBA,
    title_bar: gdk::RGBA,
    accent: gdk::RGBA,
    text: gdk::RGBA,
}

impl Palette {
    fn from_look(look: &MockLook) -> Self {
        Self {
            layout: look.layout,
            wallpaper: parse_color(&look.wallpaper),
            glow: parse_color(&look.glow),
            bar: parse_color(&look.bar),
            window: parse_color(&look.window),
            title_bar: parse_color(&look.title_bar),
            accent: parse_color(&look.accent),
            text: parse_color(&look.text),
        }
    }
}

fn parse_color(value: &str) -> gdk::RGBA {
    gdk::RGBA::parse(value).unwrap_or(gdk::RGBA::BLACK)
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct ScreenshotMock {
        pub(super) palette: Cell<Option<Palette>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScreenshotMock {
        const NAME: &'static str = "ScreenshotMock";
        type Type = super::ScreenshotMock;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("screenshotmock");
            klass.set_accessible_role(gtk::AccessibleRole::Img);
        }
    }

    impl ObjectImpl for ScreenshotMock {}

    impl WidgetImpl for ScreenshotMock {
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
            let Some(palette) = self.palette.get() else {
                return;
            };
            let widget = self.obj();
            let width = widget.width() as f32;
            let height = widget.height() as f32;
            let bounds = graphene::Rect::new(0.0, 0.0, width, height);

            snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(bounds, CORNER_RADIUS));
            snapshot.append_color(&palette.wallpaper, &bounds);
            draw_glow(snapshot, &palette, width, height);
            draw_top_bar(snapshot, &palette, width, height);

            match palette.layout {
                MockLayout::Tiled => draw_tiled_windows(snapshot, &palette, width, height),
                MockLayout::Floating => draw_floating_windows(snapshot, &palette, width, height),
            }

            snapshot.pop();
        }
    }
}

glib::wrapper! {
    pub struct ScreenshotMock(ObjectSubclass<imp::ScreenshotMock>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ScreenshotMock {
    pub fn set_look(&self, look: &MockLook) {
        self.imp().palette.set(Some(Palette::from_look(look)));
        self.queue_draw();
    }
}

fn fill_rounded(snapshot: &gtk::Snapshot, rect: graphene::Rect, radius: f32, color: &gdk::RGBA) {
    snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(rect, radius));
    snapshot.append_color(color, &rect);
    snapshot.pop();
}

/// A large soft circle in the top right corner of the wallpaper.
fn draw_glow(snapshot: &gtk::Snapshot, palette: &Palette, width: f32, height: f32) {
    let size = width * 0.7;
    let rect = graphene::Rect::new(width * 0.42, -height * 0.3, size, size);
    fill_rounded(snapshot, rect, size / 2.0, &palette.glow);
}

fn draw_top_bar(snapshot: &gtk::Snapshot, palette: &Palette, width: f32, height: f32) {
    let rect = graphene::Rect::new(0.0, 0.0, width, height * 0.06);
    snapshot.append_color(&palette.bar, &rect);
}

/// One large window on the left with an accent border, two stacked on the right.
fn draw_tiled_windows(snapshot: &gtk::Snapshot, palette: &Palette, width: f32, height: f32) {
    let main = graphene::Rect::new(width * 0.03, height * 0.1, width * 0.55, height * 0.86);
    fill_rounded(snapshot, main, WINDOW_RADIUS, &palette.window);

    let outline = gsk::RoundedRect::from_rect(main, WINDOW_RADIUS);
    snapshot.append_border(&outline, &[2.0; 4], &[palette.accent; 4]);

    draw_text_lines(snapshot, palette, main);

    let top_right = graphene::Rect::new(width * 0.61, height * 0.1, width * 0.36, height * 0.41);
    let bottom_right = graphene::Rect::new(width * 0.61, height * 0.55, width * 0.36, height * 0.41);
    fill_rounded(snapshot, top_right, WINDOW_RADIUS, &palette.window);
    fill_rounded(snapshot, bottom_right, WINDOW_RADIUS, &palette.window);
}

/// Three short lines, like a terminal prompt, inside a window.
fn draw_text_lines(snapshot: &gtk::Snapshot, palette: &Palette, window: graphene::Rect) {
    let lines = [(0.7, palette.accent), (0.45, palette.text), (0.6, palette.text)];
    let x = window.x() + window.width() * 0.08;
    let line_height = (window.height() * 0.04).max(3.0);

    for (index, (length, color)) in lines.iter().enumerate() {
        let y = window.y() + window.height() * 0.1 + index as f32 * line_height * 2.5;
        let rect = graphene::Rect::new(x, y, window.width() * length * 0.8, line_height);
        fill_rounded(snapshot, rect, line_height / 2.0, color);
    }
}

/// Two overlapping windows with title bars, and a dock at the bottom.
fn draw_floating_windows(snapshot: &gtk::Snapshot, palette: &Palette, width: f32, height: f32) {
    let back = graphene::Rect::new(width * 0.08, height * 0.16, width * 0.5, height * 0.58);
    let front = graphene::Rect::new(width * 0.44, height * 0.32, width * 0.46, height * 0.5);
    draw_floating_window(snapshot, palette, back);
    draw_floating_window(snapshot, palette, front);

    let dock = graphene::Rect::new(width * 0.33, height * 0.87, width * 0.34, height * 0.09);
    fill_rounded(snapshot, dock, height * 0.03, &DOCK_COLOR);
}

fn draw_floating_window(snapshot: &gtk::Snapshot, palette: &Palette, rect: graphene::Rect) {
    let outline = gsk::RoundedRect::from_rect(rect, WINDOW_RADIUS);
    snapshot.append_outset_shadow(&outline, &WINDOW_SHADOW, 0.0, 6.0, 0.0, 18.0);

    snapshot.push_rounded_clip(&outline);
    snapshot.append_color(&palette.window, &rect);
    let title_bar = graphene::Rect::new(rect.x(), rect.y(), rect.width(), rect.height() * 0.16);
    snapshot.append_color(&palette.title_bar, &title_bar);
    snapshot.pop();
}
