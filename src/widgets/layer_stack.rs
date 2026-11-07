//! The Your System stack: one flat bar per layer of the system, from hardware
//! at the bottom to the user's apps at the top, each with its logo and name.
//! Lower layers are wider, so the stack reads as a pyramid that holds the
//! layers above it.

use std::cell::RefCell;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene, gsk, pango};

use super::edge::drop_shadow;

const BAR_WIDTH: f32 = 340.0;
const BAR_HEIGHT: f32 = 56.0;
const BAR_GAP: f32 = 10.0;
const BAR_RADIUS: f32 = 16.0;
/// Each layer is this much narrower than the one below it.
const NARROWING: f32 = 26.0;
const ICON_TILE_SIZE: f32 = 40.0;
const ICON_SCALE: f32 = 0.55;
const PADDING: f32 = 8.0;
const LABEL_GAP: f32 = 12.0;
const ICON_TILE: gdk::RGBA = gdk::RGBA::new(0.0, 0.0, 0.02, 0.18);
const LIGHT_TEXT: gdk::RGBA = gdk::RGBA::new(1.0, 1.0, 1.0, 0.95);
const DARK_TEXT: gdk::RGBA = gdk::RGBA::new(0.0, 0.0, 0.02, 0.85);
/// Above this brightness, white text gets hard to read; use dark text instead.
const LIGHT_COLOR_LUMINANCE: f32 = 0.55;
/// Small app tiles drawn on the top bar.
const APP_TILE_COLORS: [gdk::RGBA; 3] = [
    gdk::RGBA::new(0.21, 0.52, 0.89, 1.0),
    gdk::RGBA::new(0.90, 0.38, 0.0, 1.0),
    gdk::RGBA::new(0.15, 0.64, 0.41, 1.0),
];

/// One layer of the stack.
#[derive(Debug, Clone)]
pub struct StackLayer {
    pub label: String,
    pub color: gdk::RGBA,
    /// A symbolic icon drawn on the bar, such as a distro logo.
    pub icon_name: Option<String>,
    /// Draws small app tiles on the bar instead of an icon.
    pub shows_apps: bool,
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct LayerStack {
        pub layers: RefCell<Vec<StackLayer>>,
        pub glow: RefCell<Option<gdk::RGBA>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LayerStack {
        const NAME: &'static str = "LayerStack";
        type Type = super::LayerStack;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("layerstack");
            klass.set_accessible_role(gtk::AccessibleRole::Img);
        }
    }

    impl ObjectImpl for LayerStack {}

    impl WidgetImpl for LayerStack {
        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                gtk::Orientation::Horizontal => (0, 420, -1, -1),
                _ => (340, 340, -1, -1),
            }
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let layers = self.layers.borrow();
            if layers.is_empty() {
                return;
            }
            let bars = bar_rects(layers.len(), widget.width() as f32, widget.height() as f32);

            if let Some(glow) = *self.glow.borrow() {
                draw_glow(snapshot, &bars, &glow);
            }
            for (layer, bar) in layers.iter().zip(&bars) {
                draw_bar(snapshot, &widget, layer, bar);
            }
        }
    }
}

glib::wrapper! {
    pub struct LayerStack(ObjectSubclass<imp::LayerStack>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl LayerStack {
    /// `layers` go from the bottom layer to the top one. `glow` lights the area behind them.
    pub fn set_layers(&self, layers: Vec<StackLayer>, glow: gdk::RGBA) {
        let description = layers
            .iter()
            .map(|layer| layer.label.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        self.update_property(&[gtk::accessible::Property::Label(&description)]);

        let imp = self.imp();
        imp.layers.replace(layers);
        imp.glow.replace(Some(glow));
        self.queue_draw();
    }
}

/// Where each bar goes, from the bottom layer up, centered in the widget.
fn bar_rects(count: usize, width: f32, height: f32) -> Vec<graphene::Rect> {
    let total_height = count as f32 * BAR_HEIGHT + (count as f32 - 1.0) * BAR_GAP;
    let bottom_width = BAR_WIDTH.min(width - 2.0 * PADDING);
    let bottom_y = (height + total_height) / 2.0 - BAR_HEIGHT;

    (0..count)
        .map(|index| {
            let bar_width = bottom_width - index as f32 * NARROWING;
            let x = (width - bar_width) / 2.0;
            let y = bottom_y - index as f32 * (BAR_HEIGHT + BAR_GAP);
            graphene::Rect::new(x, y, bar_width, BAR_HEIGHT)
        })
        .collect()
}

/// A soft colored light behind the whole stack.
fn draw_glow(snapshot: &gtk::Snapshot, bars: &[graphene::Rect], color: &gdk::RGBA) {
    let Some(area) = bars.iter().copied().reduce(|first, second| first.union(&second)) else {
        return;
    };
    let rect = area.inset_r(-24.0, -12.0);
    let glow = gdk::RGBA::new(color.red(), color.green(), color.blue(), 0.45);

    snapshot.push_blur(40.0);
    snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(rect, rect.height() / 2.0));
    snapshot.append_color(&glow, &rect);
    snapshot.pop();
    snapshot.pop();
}

fn draw_bar(snapshot: &gtk::Snapshot, widget: &LayerStack, layer: &StackLayer, bar: &graphene::Rect) {
    drop_shadow(snapshot, bar, BAR_RADIUS, 4.0);
    snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(*bar, BAR_RADIUS));
    snapshot.append_color(&layer.color, bar);
    snapshot.pop();

    let tile = graphene::Rect::new(bar.x() + PADDING, bar.y() + PADDING, ICON_TILE_SIZE, ICON_TILE_SIZE);
    if layer.shows_apps {
        draw_app_tiles(snapshot, &tile);
    } else {
        draw_icon_tile(snapshot, widget, layer.icon_name.as_deref(), &tile);
    }

    let text_color = text_color_on(&layer.color);
    draw_label(snapshot, widget, &layer.label, tile.x() + tile.width() + LABEL_GAP, bar, &text_color);
}

/// A darker square with the layer's logo, or an empty one when it has none.
fn draw_icon_tile(snapshot: &gtk::Snapshot, widget: &LayerStack, icon_name: Option<&str>, tile: &graphene::Rect) {
    snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(*tile, BAR_RADIUS - PADDING / 2.0));
    snapshot.append_color(&ICON_TILE, tile);
    snapshot.pop();

    let Some(icon_name) = icon_name else {
        return;
    };
    let icon_theme = gtk::IconTheme::for_display(&widget.display());
    if !icon_theme.has_icon(icon_name) {
        return;
    }

    let size = ICON_TILE_SIZE * ICON_SCALE;
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
        tile.x() + (ICON_TILE_SIZE - size) / 2.0,
        tile.y() + (ICON_TILE_SIZE - size) / 2.0,
    ));
    paintable.snapshot_symbolic(snapshot, f64::from(size), f64::from(size), &[gdk::RGBA::WHITE]);
    snapshot.restore();
}

/// Three small app tiles in a 2 by 2 grid with one gap, like an app grid.
fn draw_app_tiles(snapshot: &gtk::Snapshot, tile: &graphene::Rect) {
    let size = (ICON_TILE_SIZE - 4.0) / 2.0;
    let spots = [(0.0, 0.0), (size + 4.0, 0.0), (0.0, size + 4.0)];

    for (color, (dx, dy)) in APP_TILE_COLORS.iter().zip(spots) {
        let rect = graphene::Rect::new(tile.x() + dx, tile.y() + dy, size, size);
        snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(rect, size * 0.3));
        snapshot.append_color(color, &rect);
        snapshot.pop();
    }
}

fn draw_label(
    snapshot: &gtk::Snapshot,
    widget: &LayerStack,
    text: &str,
    x: f32,
    bar: &graphene::Rect,
    color: &gdk::RGBA,
) {
    let layout = widget.create_pango_layout(Some(text));
    let mut font = widget.pango_context().font_description().unwrap_or_default();
    font.set_weight(pango::Weight::Bold);
    layout.set_font_description(Some(&font));
    layout.set_ellipsize(pango::EllipsizeMode::End);
    let available = bar.x() + bar.width() - x - PADDING * 2.0;
    layout.set_width((available.max(0.0) * pango::SCALE as f32) as i32);
    let (_, extents) = layout.pixel_extents();

    snapshot.save();
    snapshot.translate(&graphene::Point::new(x, bar.y() + (bar.height() - extents.height() as f32) / 2.0));
    snapshot.append_layout(&layout, color);
    snapshot.restore();
}

/// White on dark and saturated colors, dark on light ones like yellow.
fn text_color_on(color: &gdk::RGBA) -> gdk::RGBA {
    let luminance = 0.2126 * color.red() + 0.7152 * color.green() + 0.0722 * color.blue();
    if luminance > LIGHT_COLOR_LUMINANCE {
        DARK_TEXT
    } else {
        LIGHT_TEXT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lower_layers_are_wider_and_lower() {
        let bars = bar_rects(3, 500.0, 340.0);
        assert!(bars[0].width() > bars[1].width());
        assert!(bars[0].y() > bars[1].y());
    }

    #[test]
    fn light_colors_get_dark_text() {
        let yellow = gdk::RGBA::new(0.97, 0.89, 0.36, 1.0);
        let blue = gdk::RGBA::new(0.11, 0.44, 0.85, 1.0);
        assert_eq!(text_color_on(&yellow), DARK_TEXT);
        assert_eq!(text_color_on(&blue), LIGHT_TEXT);
    }
}
