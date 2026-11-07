//! The Your System stack: one isometric slab per layer of the system, from
//! hardware at the bottom to the user's apps at the top, each with a label.

use std::cell::RefCell;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene, gsk, pango};

use super::iso::{apply_floor_transform, floor_to_screen, screen_drop_in_floor};

/// The side of each square slab, in floor units.
const SLAB_SIZE: f32 = 150.0;
const SLAB_RADIUS: f32 = 22.0;
const SLAB_THICKNESS: f32 = 16.0;
/// How far each slab sits above the one below it, on screen.
const LAYER_STEP: f32 = 60.0;
const LABEL_GAP: f32 = 22.0;
const LOGO_SCALE: f32 = 0.3;
/// Where the logo sits on the slab, in floor units from its center: toward the
/// front corner, the part the slab above does not cover.
const LOGO_OFFSET: (f32, f32) = (-0.32, 0.5);
const EDGE_HIGHLIGHT: gdk::RGBA = gdk::RGBA::new(1.0, 1.0, 1.0, 0.35);
const CONNECTOR: gdk::RGBA = gdk::RGBA::new(1.0, 1.0, 1.0, 0.3);
const LABEL_COLOR: gdk::RGBA = gdk::RGBA::new(1.0, 1.0, 1.0, 0.92);
/// Small app tiles drawn on the top slab.
const APP_TILE_COLORS: [gdk::RGBA; 3] = [
    gdk::RGBA::new(0.21, 0.52, 0.89, 1.0),
    gdk::RGBA::new(0.90, 0.38, 0.0, 1.0),
    gdk::RGBA::new(0.15, 0.64, 0.41, 1.0),
];

/// One slab of the stack.
#[derive(Debug, Clone)]
pub struct StackLayer {
    pub label: String,
    pub color: gdk::RGBA,
    /// A symbolic icon drawn on the slab, such as a distro logo.
    pub icon_name: Option<String>,
    /// Draws small app tiles on the slab instead of an icon.
    pub shows_apps: bool,
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct IsoStack {
        pub layers: RefCell<Vec<StackLayer>>,
        pub glow: RefCell<Option<gdk::RGBA>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IsoStack {
        const NAME: &'static str = "IsoStack";
        type Type = super::IsoStack;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("isostack");
            klass.set_accessible_role(gtk::AccessibleRole::Img);
        }
    }

    impl ObjectImpl for IsoStack {}

    impl WidgetImpl for IsoStack {
        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                gtk::Orientation::Horizontal => (0, 480, -1, -1),
                _ => (430, 430, -1, -1),
            }
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let layers = self.layers.borrow();
            if layers.is_empty() {
                return;
            }

            let base = graphene::Point::new(widget.width() as f32 * 0.3, widget.height() as f32 * 0.75);
            if let Some(glow) = *self.glow.borrow() {
                draw_glow(snapshot, base, &glow);
            }

            for (index, layer) in layers.iter().enumerate() {
                let origin = graphene::Point::new(base.x(), base.y() - index as f32 * LAYER_STEP);
                draw_slab(snapshot, &widget, origin, layer);
                draw_label(snapshot, &widget, origin, &layer.label);
            }
        }
    }
}

glib::wrapper! {
    pub struct IsoStack(ObjectSubclass<imp::IsoStack>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl IsoStack {
    /// `layers` go from the bottom slab to the top one. `glow` lights the floor under them.
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

/// A soft colored light on the floor under the stack.
fn draw_glow(snapshot: &gtk::Snapshot, base: graphene::Point, color: &gdk::RGBA) {
    let width = SLAB_SIZE * 2.4;
    let height = width * 0.5;
    let rect = graphene::Rect::new(base.x() - width / 2.0, base.y() - height / 2.0 + 20.0, width, height);
    let glow = gdk::RGBA::new(color.red(), color.green(), color.blue(), 0.55);

    snapshot.push_blur(36.0);
    snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(rect, height / 2.0));
    snapshot.append_color(&glow, &rect);
    snapshot.pop();
    snapshot.pop();
}

/// A square slab with a darker edge below it and a light rim on top.
fn draw_slab(snapshot: &gtk::Snapshot, widget: &IsoStack, origin: graphene::Point, layer: &StackLayer) {
    let half = SLAB_SIZE / 2.0;
    let top = graphene::Rect::new(-half, -half, SLAB_SIZE, SLAB_SIZE);
    let (drop_x, drop_y) = screen_drop_in_floor(SLAB_THICKNESS);
    let edge = graphene::Rect::new(-half + drop_x, -half + drop_y, SLAB_SIZE, SLAB_SIZE);
    let outline = gsk::RoundedRect::from_rect(top, SLAB_RADIUS);

    snapshot.save();
    snapshot.translate(&origin);
    apply_floor_transform(snapshot);

    fill_rounded(snapshot, edge, &darken(layer.color, 0.35));
    fill_rounded(snapshot, top, &layer.color);
    snapshot.append_border(&outline, &[1.5; 4], &[EDGE_HIGHLIGHT; 4]);

    if layer.shows_apps {
        draw_app_tiles(snapshot);
    } else if let Some(icon_name) = &layer.icon_name {
        draw_icon(snapshot, widget, icon_name);
    }

    snapshot.restore();
}

/// Three small app tiles in a row, centered on the slab.
fn draw_app_tiles(snapshot: &gtk::Snapshot) {
    let size = SLAB_SIZE * 0.2;
    let gap = size * 0.4;
    let start = -(size * 3.0 + gap * 2.0) / 2.0;

    for (index, color) in APP_TILE_COLORS.iter().enumerate() {
        let x = start + index as f32 * (size + gap);
        let rect = graphene::Rect::new(x, -size / 2.0, size, size);
        snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(rect, size * 0.25));
        snapshot.append_color(color, &rect);
        snapshot.pop();
    }
}

/// A white icon lying on the slab, so it takes the slab's perspective.
fn draw_icon(snapshot: &gtk::Snapshot, widget: &IsoStack, icon_name: &str) {
    let icon_theme = gtk::IconTheme::for_display(&widget.display());
    if !icon_theme.has_icon(icon_name) {
        return;
    }

    let size = SLAB_SIZE * LOGO_SCALE;
    let paintable = icon_theme.lookup_icon(
        icon_name,
        &[],
        size.round() as i32,
        widget.scale_factor(),
        gtk::TextDirection::None,
        gtk::IconLookupFlags::FORCE_SYMBOLIC,
    );

    let center_x = SLAB_SIZE / 2.0 * LOGO_OFFSET.0;
    let center_y = SLAB_SIZE / 2.0 * LOGO_OFFSET.1;

    snapshot.save();
    snapshot.translate(&graphene::Point::new(center_x - size / 2.0, center_y - size / 2.0));
    paintable.snapshot_symbolic(snapshot, f64::from(size), f64::from(size), &[gdk::RGBA::WHITE]);
    snapshot.restore();
}

/// The layer name to the right of the slab, joined to its corner by a short line.
fn draw_label(snapshot: &gtk::Snapshot, widget: &IsoStack, origin: graphene::Point, text: &str) {
    let corner = right_corner(origin);
    let label_x = corner.x() + LABEL_GAP;

    let connector = graphene::Rect::new(corner.x() + 6.0, corner.y(), LABEL_GAP - 10.0, 1.5);
    snapshot.append_color(&CONNECTOR, &connector);

    let layout = widget.create_pango_layout(Some(text));
    let mut font = widget.pango_context().font_description().unwrap_or_default();
    font.set_weight(pango::Weight::Bold);
    layout.set_font_description(Some(&font));
    let (_, extents) = layout.pixel_extents();

    snapshot.save();
    snapshot.translate(&graphene::Point::new(label_x, corner.y() - extents.height() as f32 / 2.0));
    snapshot.append_layout(&layout, &LABEL_COLOR);
    snapshot.restore();
}

/// The slab corner furthest to the right on screen.
fn right_corner(origin: graphene::Point) -> graphene::Point {
    let half = SLAB_SIZE / 2.0;
    let corners = [(-half, -half), (half, -half), (half, half), (-half, half)];

    let rightmost = corners
        .iter()
        .map(|(x, y)| floor_to_screen(*x, *y))
        .max_by(|a, b| a.x().total_cmp(&b.x()))
        .unwrap_or(graphene::Point::new(0.0, 0.0));

    graphene::Point::new(origin.x() + rightmost.x(), origin.y() + rightmost.y())
}

fn fill_rounded(snapshot: &gtk::Snapshot, rect: graphene::Rect, color: &gdk::RGBA) {
    snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(rect, SLAB_RADIUS));
    snapshot.append_color(color, &rect);
    snapshot.pop();
}

/// `color` with `amount` (0 to 1) of black mixed in, for slab edges.
pub fn darken(color: gdk::RGBA, amount: f32) -> gdk::RGBA {
    let keep = 1.0 - amount;
    gdk::RGBA::new(color.red() * keep, color.green() * keep, color.blue() * keep, color.alpha())
}
