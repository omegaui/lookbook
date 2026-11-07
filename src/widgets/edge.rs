//! The flat "off the edge" layout shared by banner art: a large screen, seen
//! straight on, that starts near the top-left of its area and runs off the
//! right and bottom edges, like the featured banners in GNOME Software.

use gtk::prelude::*;
use gtk::{gdk, graphene, gsk};

/// The gap between the area's top-left corner and the screen.
const INSET_X: f32 = 40.0;
const INSET_Y: f32 = 40.0;
/// The screen is at least this wide, so it always runs off a narrow area.
const MIN_SCREEN_WIDTH: f32 = 640.0;
/// How much wider than the area the screen is, so it runs off a wide one too.
const OVERFLOW: f32 = 1.15;
/// Screenshots are 16:10 or 16:9; 16:10 crops either one well.
const ASPECT: f32 = 10.0 / 16.0;
pub const SCREEN_RADIUS: f32 = 16.0;
const SHADOW: gdk::RGBA = gdk::RGBA::new(0.0, 0.0, 0.02, 0.3);

/// Where the screen goes in an area of `width` by `height`.
pub fn screen_rect(width: f32) -> graphene::Rect {
    let screen_width = MIN_SCREEN_WIDTH.max(width * OVERFLOW);
    graphene::Rect::new(INSET_X, INSET_Y, screen_width, screen_width * ASPECT)
}

/// The width `screen_rect` was designed around, for art drawn in fixed units.
pub fn design_width() -> f32 {
    MIN_SCREEN_WIDTH
}

/// A soft shadow under a rounded rectangle, so it seems `lift` pixels above
/// the background: higher means a lower, blurrier shadow.
pub fn drop_shadow(snapshot: &gtk::Snapshot, rect: &graphene::Rect, radius: f32, lift: f32) {
    let outline = gsk::RoundedRect::from_rect(*rect, radius);
    snapshot.append_outset_shadow(&outline, &SHADOW, 0.0, lift, 0.0, lift * 3.0);
}
