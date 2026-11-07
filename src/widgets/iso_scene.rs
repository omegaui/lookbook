//! An isometric desktop scene for the spotlight carousel: a floor slab with
//! app windows standing on it, laid out the way the featured item arranges them.
//!
//! The scene is drawn on a floor seen from above: turned by -28° and flattened
//! to 64% height. That is a flat 2D transform, so text-free shapes stay sharp.

use std::cell::Cell;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene, gsk};

use super::iso::{apply_floor_transform, screen_drop_in_floor};
use crate::catalog::Accent;

/// How the windows sit on the floor.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SceneLayout {
    /// Columns on an endless strip, like niri.
    Scrolling,
    /// One large window and two stacked beside it, like sway or Hyprland.
    Tiling,
    /// Overlapping windows with a top bar and a dock, like GNOME.
    #[default]
    Floating,
    /// Overlapping windows over a full-width bottom panel, like KDE Plasma.
    Panel,
}

const FLOOR_THICKNESS: f32 = 14.0;
const WINDOW_THICKNESS: f32 = 8.0;
const WINDOW_RADIUS: f32 = 12.0;
const TITLE_BAR_HEIGHT: f32 = 18.0;

const LIGHT_FACE: gdk::RGBA = rgb(0xfa, 0xfa, 0xfb);
const LIGHT_TITLE: gdk::RGBA = rgb(0xeb, 0xeb, 0xed);
const LIGHT_EDGE: gdk::RGBA = rgb(0xb0, 0xaf, 0xb5);
const DARK_FACE: gdk::RGBA = rgb(0x24, 0x1f, 0x31);
const DARK_TITLE: gdk::RGBA = rgb(0x3d, 0x38, 0x46);
const DARK_EDGE: gdk::RGBA = rgb(0x0d, 0x0b, 0x12);
const GRAY_LINE: gdk::RGBA = rgb(0xde, 0xdd, 0xda);
const SOFT_GRAY: gdk::RGBA = rgb(0xf0, 0xf0, 0xf3);
const TERMINAL_GREEN: gdk::RGBA = rgb(0x57, 0xe3, 0x89);
const TERMINAL_BLUE: gdk::RGBA = rgb(0x99, 0xc1, 0xf1);
const TERMINAL_GRAY: gdk::RGBA = rgb(0x77, 0x76, 0x7b);
const CODE_PURPLE: gdk::RGBA = rgb(0x91, 0x41, 0xac);
const CODE_BLUE: gdk::RGBA = rgb(0x1c, 0x71, 0xd8);
const CODE_ORANGE: gdk::RGBA = rgb(0xe6, 0x61, 0x00);
const CLOSE_BUTTON: gdk::RGBA = rgb(0xc0, 0xbf, 0xbc);
const TOP_BAR: gdk::RGBA = rgb(0x00, 0x00, 0x00);
const FLOOR_DOT: gdk::RGBA = gdk::RGBA::new(1.0, 1.0, 1.0, 0.25);
const DOCK: gdk::RGBA = gdk::RGBA::new(1.0, 1.0, 1.0, 0.55);

/// `base` moved `amount` (0 to 1) of the way toward `target`.
fn mix(base: gdk::RGBA, target: gdk::RGBA, amount: f32) -> gdk::RGBA {
    let blend = |from: f32, to: f32| from + (to - from) * amount;
    gdk::RGBA::new(
        blend(base.red(), target.red()),
        blend(base.green(), target.green()),
        blend(base.blue(), target.blue()),
        base.alpha(),
    )
}

const fn rgb(red: u8, green: u8, blue: u8) -> gdk::RGBA {
    gdk::RGBA::new(red as f32 / 255.0, green as f32 / 255.0, blue as f32 / 255.0, 1.0)
}

/// Colors from the GNOME palette for one accent: the accent itself, the floor, the floor's edge.
#[derive(Debug, Clone, Copy)]
struct ScenePalette {
    accent: gdk::RGBA,
    floor: gdk::RGBA,
    floor_edge: gdk::RGBA,
}

impl ScenePalette {
    /// Builds the floor from any brand color: a lighter top and a darker edge.
    fn from_color(color: gdk::RGBA) -> Self {
        Self {
            accent: color,
            floor: mix(color, gdk::RGBA::WHITE, 0.45),
            floor_edge: mix(color, gdk::RGBA::BLACK, 0.3),
        }
    }

    fn for_accent(accent: Accent) -> Self {
        let (accent, floor, floor_edge) = match accent {
            Accent::Blue => (rgb(0x35, 0x84, 0xe4), rgb(0x62, 0xa0, 0xea), rgb(0x1a, 0x5f, 0xb4)),
            Accent::Green => (rgb(0x2e, 0xc2, 0x7e), rgb(0x8f, 0xf0, 0xa4), rgb(0x26, 0xa2, 0x69)),
            Accent::Yellow => (rgb(0xe5, 0xa5, 0x0a), rgb(0xf9, 0xf0, 0x6b), rgb(0xc8, 0x88, 0x00)),
            Accent::Orange => (rgb(0xe6, 0x61, 0x00), rgb(0xff, 0xa3, 0x48), rgb(0xc6, 0x46, 0x00)),
            Accent::Red => (rgb(0xe0, 0x1b, 0x24), rgb(0xf6, 0x61, 0x51), rgb(0xa5, 0x1d, 0x2d)),
            Accent::Purple => (rgb(0x81, 0x3d, 0x9c), rgb(0xc0, 0x61, 0xcb), rgb(0x61, 0x35, 0x83)),
            Accent::Brown => (rgb(0x86, 0x5e, 0x3c), rgb(0xcd, 0xab, 0x8f), rgb(0x63, 0x45, 0x2c)),
            Accent::Dark => (rgb(0x3d, 0x38, 0x46), rgb(0x77, 0x76, 0x7b), rgb(0x24, 0x1f, 0x31)),
        };
        Self { accent, floor, floor_edge }
    }
}

/// What an app window shows.
#[derive(Debug, Clone, Copy)]
enum WindowKind {
    Terminal,
    Files,
    Browser,
    Editor,
    Music,
}

/// One window on the floor, in floor coordinates.
#[derive(Debug, Clone, Copy)]
struct WindowSpec {
    rect: graphene::Rect,
    kind: WindowKind,
    focused: bool,
}

fn window(x: f32, y: f32, width: f32, height: f32, kind: WindowKind) -> WindowSpec {
    WindowSpec {
        rect: graphene::Rect::new(x, y, width, height),
        kind,
        focused: false,
    }
}

fn focused(spec: WindowSpec) -> WindowSpec {
    WindowSpec { focused: true, ..spec }
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct IsoScene {
        pub layout: Cell<SceneLayout>,
        pub(super) palette: Cell<Option<ScenePalette>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IsoScene {
        const NAME: &'static str = "IsoScene";
        type Type = super::IsoScene;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("isoscene");
            klass.set_accessible_role(gtk::AccessibleRole::Presentation);
        }
    }

    impl ObjectImpl for IsoScene {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_overflow(gtk::Overflow::Hidden);
        }
    }

    impl WidgetImpl for IsoScene {
        /// It can shrink to nothing; the scene is cropped, never squeezed.
        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                gtk::Orientation::Horizontal => (0, 480, -1, -1),
                _ => (300, 340, -1, -1),
            }
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let palette = self
                .palette
                .get()
                .unwrap_or_else(|| ScenePalette::for_accent(Accent::Blue));
            let widget = self.obj();
            let width = widget.width() as f32;
            let height = widget.height() as f32;

            snapshot.save();
            snapshot.translate(&graphene::Point::new(width * 0.06, height * 0.58));
            apply_floor_transform(snapshot);

            match self.layout.get() {
                SceneLayout::Scrolling => draw_scrolling_scene(snapshot, &palette),
                SceneLayout::Tiling => draw_tiling_scene(snapshot, &palette),
                SceneLayout::Floating => draw_floating_scene(snapshot, &palette),
                SceneLayout::Panel => draw_panel_scene(snapshot, &palette),
            }

            snapshot.restore();
        }
    }
}

glib::wrapper! {
    pub struct IsoScene(ObjectSubclass<imp::IsoScene>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl IsoScene {
    pub fn set_scene(&self, layout: SceneLayout, accent: Accent) {
        self.show(layout, ScenePalette::for_accent(accent));
    }

    /// Like `set_scene`, with the floor colored from a brand color such as "#1793d1".
    pub fn set_scene_color(&self, layout: SceneLayout, color: &str) {
        let color = gdk::RGBA::parse(color).unwrap_or(rgb(0x35, 0x84, 0xe4));
        self.show(layout, ScenePalette::from_color(color));
    }

    fn show(&self, layout: SceneLayout, palette: ScenePalette) {
        let imp = self.imp();
        imp.layout.set(layout);
        imp.palette.set(Some(palette));
        self.queue_draw();
    }
}

/// niri: windows in columns along a long strip that runs off the edge.
fn draw_scrolling_scene(snapshot: &gtk::Snapshot, palette: &ScenePalette) {
    draw_floor(snapshot, palette, graphene::Rect::new(-160.0, -30.0, 1200.0, 300.0));

    let windows = [
        window(-120.0, 0.0, 150.0, 240.0, WindowKind::Files),
        focused(window(50.0, 0.0, 240.0, 240.0, WindowKind::Terminal)),
        window(310.0, 0.0, 160.0, 111.0, WindowKind::Music),
        window(310.0, 129.0, 160.0, 111.0, WindowKind::Editor),
        window(490.0, 0.0, 200.0, 240.0, WindowKind::Browser),
        window(710.0, 0.0, 160.0, 240.0, WindowKind::Files),
    ];
    draw_windows(snapshot, palette, &windows);
}

/// Tiling: one large window and two stacked beside it.
fn draw_tiling_scene(snapshot: &gtk::Snapshot, palette: &ScenePalette) {
    draw_floor(snapshot, palette, graphene::Rect::new(0.0, -40.0, 620.0, 380.0));

    let windows = [
        focused(window(20.0, -20.0, 300.0, 340.0, WindowKind::Terminal)),
        window(336.0, -20.0, 264.0, 162.0, WindowKind::Browser),
        window(336.0, 158.0, 264.0, 162.0, WindowKind::Editor),
    ];
    draw_windows(snapshot, palette, &windows);
}

/// Floating: a top bar, two overlapping windows and a dock.
fn draw_floating_scene(snapshot: &gtk::Snapshot, palette: &ScenePalette) {
    let floor = graphene::Rect::new(0.0, -60.0, 640.0, 400.0);
    draw_floor(snapshot, palette, floor);

    let top_bar = graphene::Rect::new(floor.x() + 14.0, floor.y() + 12.0, floor.width() - 28.0, 16.0);
    fill_rounded(snapshot, top_bar, 8.0, &TOP_BAR);

    let windows = [
        window(40.0, -10.0, 340.0, 220.0, WindowKind::Browser),
        focused(window(250.0, 80.0, 330.0, 200.0, WindowKind::Files)),
    ];
    draw_windows(snapshot, palette, &windows);

    draw_dock(snapshot, palette, graphene::Rect::new(220.0, 292.0, 200.0, 34.0));
}

/// KDE Plasma and similar: overlapping windows and a full-width bottom panel.
fn draw_panel_scene(snapshot: &gtk::Snapshot, palette: &ScenePalette) {
    let floor = graphene::Rect::new(0.0, -60.0, 640.0, 400.0);
    draw_floor(snapshot, palette, floor);

    let windows = [
        window(40.0, -40.0, 340.0, 220.0, WindowKind::Browser),
        focused(window(250.0, 50.0, 330.0, 200.0, WindowKind::Files)),
    ];
    draw_windows(snapshot, palette, &windows);

    let panel = graphene::Rect::new(floor.x() + 14.0, floor.y() + floor.height() - 52.0, floor.width() - 28.0, 38.0);
    fill_rounded(snapshot, panel, 10.0, &TOP_BAR);

    let launcher = graphene::Rect::new(panel.x() + 10.0, panel.y() + 8.0, 22.0, 22.0);
    fill_rounded(snapshot, launcher, 6.0, &palette.accent);
    for index in 0..3 {
        let task = graphene::Rect::new(panel.x() + 48.0 + index as f32 * 32.0, panel.y() + 8.0, 22.0, 22.0);
        fill_rounded(snapshot, task, 6.0, &DOCK);
    }
    let clock = graphene::Rect::new(panel.x() + panel.width() - 70.0, panel.y() + 14.0, 56.0, 10.0);
    fill_rounded(snapshot, clock, 5.0, &DOCK);
}

/// The ground the windows stand on, with a visible edge and a dot grid.
fn draw_floor(snapshot: &gtk::Snapshot, palette: &ScenePalette, rect: graphene::Rect) {
    let (dx, dy) = screen_drop_in_floor(FLOOR_THICKNESS);
    fill_rounded(snapshot, offset(rect, dx, dy), 28.0, &palette.floor_edge);
    fill_rounded(snapshot, rect, 28.0, &palette.floor);

    let spacing = 40.0;
    let mut y = rect.y() + spacing / 2.0;
    while y < rect.y() + rect.height() {
        let mut x = rect.x() + spacing / 2.0;
        while x < rect.x() + rect.width() {
            fill_rounded(snapshot, graphene::Rect::new(x - 2.0, y - 2.0, 4.0, 4.0), 2.0, &FLOOR_DOT);
            x += spacing;
        }
        y += spacing;
    }
}

fn draw_windows(snapshot: &gtk::Snapshot, palette: &ScenePalette, windows: &[WindowSpec]) {
    for spec in windows {
        draw_window(snapshot, palette, spec);
    }
}

fn draw_window(snapshot: &gtk::Snapshot, palette: &ScenePalette, spec: &WindowSpec) {
    let is_dark = matches!(spec.kind, WindowKind::Terminal);
    let (face, title, edge) = if is_dark {
        (DARK_FACE, DARK_TITLE, DARK_EDGE)
    } else {
        (LIGHT_FACE, LIGHT_TITLE, LIGHT_EDGE)
    };
    let rect = spec.rect;

    // The edge below the face makes the window look like a thick tile.
    let (dx, dy) = screen_drop_in_floor(WINDOW_THICKNESS);
    fill_rounded(snapshot, offset(rect, dx, dy), WINDOW_RADIUS, &edge);

    let outline = gsk::RoundedRect::from_rect(rect, WINDOW_RADIUS);
    snapshot.push_rounded_clip(&outline);
    snapshot.append_color(&face, &rect);

    let title_bar = graphene::Rect::new(rect.x(), rect.y(), rect.width(), TITLE_BAR_HEIGHT);
    snapshot.append_color(&title, &title_bar);
    let close = graphene::Rect::new(rect.x() + rect.width() - 16.0, rect.y() + 5.0, 8.0, 8.0);
    fill_rounded(snapshot, close, 4.0, &CLOSE_BUTTON);

    let content = graphene::Rect::new(
        rect.x() + 10.0,
        rect.y() + TITLE_BAR_HEIGHT + 10.0,
        rect.width() - 20.0,
        rect.height() - TITLE_BAR_HEIGHT - 20.0,
    );
    draw_window_content(snapshot, palette, spec.kind, content);
    snapshot.pop();

    if spec.focused {
        snapshot.append_border(&outline, &[3.0; 4], &[palette.accent; 4]);
    }
}

fn draw_window_content(
    snapshot: &gtk::Snapshot,
    palette: &ScenePalette,
    kind: WindowKind,
    area: graphene::Rect,
) {
    match kind {
        WindowKind::Terminal => draw_terminal(snapshot, area),
        WindowKind::Files => draw_files(snapshot, palette, area),
        WindowKind::Browser => draw_browser(snapshot, palette, area),
        WindowKind::Editor => draw_editor(snapshot, area),
        WindowKind::Music => draw_music(snapshot, palette, area),
    }
}

/// Prompt lines in green, output in gray, a path in blue.
fn draw_terminal(snapshot: &gtk::Snapshot, area: graphene::Rect) {
    let lines = [
        (0.35, TERMINAL_GREEN),
        (0.8, TERMINAL_GRAY),
        (0.6, TERMINAL_GRAY),
        (0.45, TERMINAL_BLUE),
        (0.3, TERMINAL_GREEN),
        (0.7, TERMINAL_GRAY),
    ];
    draw_lines(snapshot, area, &lines, 14.0);
}

/// A sidebar and a grid of folders.
fn draw_files(snapshot: &gtk::Snapshot, palette: &ScenePalette, area: graphene::Rect) {
    let sidebar = graphene::Rect::new(area.x(), area.y(), area.width() * 0.26, area.height());
    fill_rounded(snapshot, sidebar, 6.0, &SOFT_GRAY);

    let grid_x = area.x() + area.width() * 0.32;
    let cell = ((area.width() * 0.68) / 3.0).min(area.height() / 2.0) - 8.0;
    for row in 0..2 {
        for column in 0..3 {
            let x = grid_x + column as f32 * (cell + 8.0);
            let y = area.y() + row as f32 * (cell + 8.0);
            fill_rounded(snapshot, graphene::Rect::new(x, y, cell, cell * 0.8), 6.0, &palette.accent);
        }
    }
}

/// An address bar, a large picture and some text.
fn draw_browser(snapshot: &gtk::Snapshot, palette: &ScenePalette, area: graphene::Rect) {
    let address = graphene::Rect::new(area.x() + area.width() * 0.2, area.y(), area.width() * 0.6, 10.0);
    fill_rounded(snapshot, address, 5.0, &SOFT_GRAY);

    let picture = graphene::Rect::new(area.x(), area.y() + 18.0, area.width(), area.height() * 0.45);
    fill_rounded(snapshot, picture, 8.0, &palette.floor);

    let text_area = graphene::Rect::new(
        area.x(),
        picture.y() + picture.height() + 10.0,
        area.width(),
        area.height() * 0.4,
    );
    draw_lines(snapshot, text_area, &[(0.9, GRAY_LINE), (0.75, GRAY_LINE), (0.5, GRAY_LINE)], 12.0);
}

/// A line-number gutter and colored code.
fn draw_editor(snapshot: &gtk::Snapshot, area: graphene::Rect) {
    let gutter = graphene::Rect::new(area.x(), area.y(), 14.0, area.height());
    fill_rounded(snapshot, gutter, 4.0, &SOFT_GRAY);

    let code = graphene::Rect::new(area.x() + 22.0, area.y(), area.width() - 22.0, area.height());
    let lines = [
        (0.5, CODE_PURPLE),
        (0.75, CODE_BLUE),
        (0.6, GRAY_LINE),
        (0.4, CODE_ORANGE),
        (0.7, GRAY_LINE),
    ];
    draw_lines(snapshot, code, &lines, 12.0);
}

/// Album art, a title and a progress bar.
fn draw_music(snapshot: &gtk::Snapshot, palette: &ScenePalette, area: graphene::Rect) {
    let art_size = area.height().min(area.width() * 0.4);
    fill_rounded(snapshot, graphene::Rect::new(area.x(), area.y(), art_size, art_size), 8.0, &palette.accent);

    let text_x = area.x() + art_size + 10.0;
    let text_width = area.width() - art_size - 10.0;
    let text = graphene::Rect::new(text_x, area.y() + 2.0, text_width, art_size);
    draw_lines(snapshot, text, &[(0.8, GRAY_LINE), (0.5, GRAY_LINE)], 12.0);

    let progress = graphene::Rect::new(text_x, area.y() + art_size - 6.0, text_width, 5.0);
    fill_rounded(snapshot, progress, 2.5, &GRAY_LINE);
    let played = graphene::Rect::new(text_x, progress.y(), text_width * 0.45, 5.0);
    fill_rounded(snapshot, played, 2.5, &palette.accent);
}

/// A translucent dock with four app icons.
fn draw_dock(snapshot: &gtk::Snapshot, palette: &ScenePalette, rect: graphene::Rect) {
    fill_rounded(snapshot, rect, 12.0, &DOCK);

    let icons = [palette.accent, TERMINAL_GREEN, CODE_ORANGE, CODE_PURPLE];
    let size = rect.height() - 12.0;
    let gap = (rect.width() - size * icons.len() as f32) / (icons.len() as f32 + 1.0);
    for (index, color) in icons.iter().enumerate() {
        let x = rect.x() + gap + index as f32 * (size + gap);
        fill_rounded(snapshot, graphene::Rect::new(x, rect.y() + 6.0, size, size), 6.0, color);
    }
}

/// Horizontal bars like lines of text, each a share of `area`'s width.
fn draw_lines(snapshot: &gtk::Snapshot, area: graphene::Rect, lines: &[(f32, gdk::RGBA)], step: f32) {
    for (index, (length, color)) in lines.iter().enumerate() {
        let y = area.y() + index as f32 * step;
        if y + 6.0 > area.y() + area.height() {
            break;
        }
        let line = graphene::Rect::new(area.x(), y, area.width() * length, 6.0);
        fill_rounded(snapshot, line, 3.0, color);
    }
}

fn fill_rounded(snapshot: &gtk::Snapshot, rect: graphene::Rect, radius: f32, color: &gdk::RGBA) {
    snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(rect, radius));
    snapshot.append_color(color, &rect);
    snapshot.pop();
}

fn offset(rect: graphene::Rect, dx: f32, dy: f32) -> graphene::Rect {
    graphene::Rect::new(rect.x() + dx, rect.y() + dy, rect.width(), rect.height())
}
