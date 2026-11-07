//! Draws a stand-in desktop screenshot from a few colors, with real app icons
//! from the icon theme. It is replaced by real screenshots once the catalog has them.

use std::cell::{Cell, RefCell};

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
const ICON_ON_LIGHT: gdk::RGBA = gdk::RGBA::new(0.37, 0.36, 0.39, 1.0);
const DOCK_ICONS: [&str; 4] = [
    "utilities-terminal-symbolic",
    "folder-symbolic",
    "web-browser-symbolic",
    "x-office-calendar-symbolic",
];

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

/// Everything a drawing step needs: where to draw, the colors, and the icons.
struct Canvas<'a> {
    snapshot: &'a gtk::Snapshot,
    palette: Palette,
    icon_theme: gtk::IconTheme,
    scale_factor: i32,
    width: f32,
    height: f32,
}

impl Canvas<'_> {
    fn fill(&self, rect: graphene::Rect, radius: f32, color: &gdk::RGBA) {
        self.snapshot
            .push_rounded_clip(&gsk::RoundedRect::from_rect(rect, radius));
        self.snapshot.append_color(color, &rect);
        self.snapshot.pop();
    }

    /// Draws a symbolic icon from the icon theme, recolored to `color`.
    fn icon(&self, name: &str, x: f32, y: f32, size: f32, color: &gdk::RGBA) {
        let pixel_size = size.round().max(1.0) as i32;
        let paintable = self.icon_theme.lookup_icon(
            name,
            &[],
            pixel_size,
            self.scale_factor,
            gtk::TextDirection::None,
            gtk::IconLookupFlags::FORCE_SYMBOLIC,
        );

        self.snapshot.save();
        self.snapshot.translate(&graphene::Point::new(x, y));
        paintable.snapshot_symbolic(self.snapshot, f64::from(size), f64::from(size), &[*color]);
        self.snapshot.restore();
    }

    /// Icons scale with the screenshot, so small cards stay readable.
    fn icon_size(&self) -> f32 {
        (self.height * 0.08).clamp(8.0, 22.0)
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct ScreenshotMock {
        pub(super) palette: Cell<Option<Palette>>,
        /// A real screenshot. Once set, it is drawn instead of the mock.
        pub(super) texture: RefCell<Option<gdk::Texture>>,
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
            let widget = self.obj();
            if let Some(texture) = self.texture.borrow().as_ref() {
                draw_real_screenshot(snapshot, texture, widget.width() as f32, widget.height() as f32);
                return;
            }
            let Some(palette) = self.palette.get() else {
                draw_placeholder(snapshot, widget.width() as f32, widget.height() as f32);
                return;
            };
            let canvas = Canvas {
                snapshot,
                palette,
                icon_theme: gtk::IconTheme::for_display(&widget.display()),
                scale_factor: widget.scale_factor(),
                width: widget.width() as f32,
                height: widget.height() as f32,
            };

            let bounds = graphene::Rect::new(0.0, 0.0, canvas.width, canvas.height);
            snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(bounds, CORNER_RADIUS));
            snapshot.append_color(&palette.wallpaper, &bounds);
            draw_glow(&canvas);

            match palette.layout {
                MockLayout::Tiled => {
                    draw_top_bar(&canvas);
                    draw_tiled_desktop(&canvas);
                }
                MockLayout::Floating => {
                    draw_top_bar(&canvas);
                    draw_floating_desktop(&canvas);
                }
                MockLayout::Panel => draw_panel_desktop(&canvas),
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

    /// Shows a real screenshot in place of the drawn one, at the same size.
    pub fn set_texture(&self, texture: &gdk::Texture) {
        self.imp().texture.replace(Some(texture.clone()));
        self.queue_draw();
    }
}

/// Scales the screenshot to cover the whole card, cropping what spills over.
fn draw_real_screenshot(snapshot: &gtk::Snapshot, texture: &gdk::Texture, width: f32, height: f32) {
    let bounds = graphene::Rect::new(0.0, 0.0, width, height);
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

    snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(bounds, CORNER_RADIUS));
    snapshot.append_texture(texture, &area);
    snapshot.pop();
}

/// A plain rounded card while a screenshot with no drawn look loads.
fn draw_placeholder(snapshot: &gtk::Snapshot, width: f32, height: f32) {
    let bounds = graphene::Rect::new(0.0, 0.0, width, height);
    let color = gdk::RGBA::new(0.5, 0.5, 0.55, 0.15);
    snapshot.push_rounded_clip(&gsk::RoundedRect::from_rect(bounds, CORNER_RADIUS));
    snapshot.append_color(&color, &bounds);
    snapshot.pop();
}

/// A large soft circle in the top right corner of the wallpaper.
fn draw_glow(canvas: &Canvas<'_>) {
    let size = canvas.width * 0.7;
    let rect = graphene::Rect::new(canvas.width * 0.42, -canvas.height * 0.3, size, size);
    canvas.fill(rect, size / 2.0, &canvas.palette.glow);
}

fn draw_top_bar(canvas: &Canvas<'_>) {
    let rect = graphene::Rect::new(0.0, 0.0, canvas.width, canvas.height * 0.06);
    canvas.snapshot.append_color(&canvas.palette.bar, &rect);
}

/// A terminal on the left, a calendar and a file manager stacked on the right.
fn draw_tiled_desktop(canvas: &Canvas<'_>) {
    let (width, height) = (canvas.width, canvas.height);
    let terminal = graphene::Rect::new(width * 0.03, height * 0.1, width * 0.55, height * 0.86);
    let calendar = graphene::Rect::new(width * 0.61, height * 0.1, width * 0.36, height * 0.41);
    let files = graphene::Rect::new(width * 0.61, height * 0.55, width * 0.36, height * 0.41);

    draw_tiled_window(canvas, terminal, true);
    draw_terminal(canvas, terminal);

    draw_tiled_window(canvas, calendar, false);
    draw_calendar(canvas, calendar, &canvas.palette.accent);

    draw_tiled_window(canvas, files, false);
    draw_folders(canvas, files, &canvas.palette.accent);
}

fn draw_tiled_window(canvas: &Canvas<'_>, rect: graphene::Rect, focused: bool) {
    canvas.fill(rect, WINDOW_RADIUS, &canvas.palette.window);
    if focused {
        let outline = gsk::RoundedRect::from_rect(rect, WINDOW_RADIUS);
        canvas
            .snapshot
            .append_border(&outline, &[2.0; 4], &[canvas.palette.accent; 4]);
    }
}

/// The terminal icon, then prompt-like lines.
fn draw_terminal(canvas: &Canvas<'_>, window: graphene::Rect) {
    let palette = &canvas.palette;
    let icon_size = canvas.icon_size();
    let x = window.x() + window.width() * 0.07;
    let top = window.y() + window.height() * 0.07;
    canvas.icon("utilities-terminal-symbolic", x, top, icon_size, &palette.accent);

    let line_height = (window.height() * 0.035).max(3.0);
    let lines = [(0.7, palette.accent), (0.45, palette.text), (0.6, palette.text), (0.35, palette.accent)];
    for (index, (length, color)) in lines.iter().enumerate() {
        let y = top + icon_size * 1.6 + index as f32 * line_height * 2.5;
        let line = graphene::Rect::new(x, y, window.width() * length * 0.8, line_height);
        canvas.fill(line, line_height / 2.0, color);
    }
}

/// A calendar icon above a month grid with today highlighted.
fn draw_calendar(canvas: &Canvas<'_>, window: graphene::Rect, icon_color: &gdk::RGBA) {
    let palette = &canvas.palette;
    let icon_size = canvas.icon_size() * 0.8;
    let padding = window.width() * 0.08;
    canvas.icon("x-office-calendar-symbolic", window.x() + padding, window.y() + padding * 0.8, icon_size, icon_color);

    let columns = 7;
    let rows = 3;
    let grid_top = window.y() + padding + icon_size * 1.3;
    let cell = (window.width() - padding * 2.0) / columns as f32;
    let dot = (cell * 0.55).min((window.y() + window.height() - grid_top) / rows as f32 * 0.6);
    let today = 10;

    for index in 0..columns * rows {
        let x = window.x() + padding + (index % columns) as f32 * cell;
        let y = grid_top + (index / columns) as f32 * cell;
        let color = if index == today { palette.accent } else { dimmed(palette.text) };
        canvas.fill(graphene::Rect::new(x, y, dot, dot), dot / 3.0, &color);
    }
}

/// Folder icons in a grid, like a file manager.
fn draw_folders(canvas: &Canvas<'_>, window: graphene::Rect, color: &gdk::RGBA) {
    let icon_size = (window.height() * 0.3).clamp(8.0, 28.0);
    let gap = (window.width() - icon_size * 3.0) / 4.0;
    let row_gap = (window.height() - icon_size * 2.0) / 3.0;

    for row in 0..2 {
        for column in 0..3 {
            let x = window.x() + gap + column as f32 * (icon_size + gap);
            let y = window.y() + row_gap + row as f32 * (icon_size + row_gap);
            canvas.icon("folder-symbolic", x, y, icon_size, color);
        }
    }
}

/// A browser behind a music player, and a dock of app icons.
fn draw_floating_desktop(canvas: &Canvas<'_>) {
    draw_browser_and_music(canvas);
    draw_dock(canvas);
}

/// KDE Plasma: no top bar, a full-width panel along the bottom.
fn draw_panel_desktop(canvas: &Canvas<'_>) {
    draw_browser_and_music(canvas);
    draw_bottom_panel(canvas);
}

fn draw_browser_and_music(canvas: &Canvas<'_>) {
    let (width, height) = (canvas.width, canvas.height);
    let browser = graphene::Rect::new(width * 0.08, height * 0.12, width * 0.5, height * 0.58);
    let music = graphene::Rect::new(width * 0.44, height * 0.28, width * 0.46, height * 0.5);

    let browser_body = draw_floating_window(canvas, browser, "web-browser-symbolic");
    draw_browser_page(canvas, browser_body);

    let music_body = draw_floating_window(canvas, music, "audio-x-generic-symbolic");
    draw_music_player(canvas, music_body);
}

/// A launcher on the left, task icons with a marker under the active one,
/// and the tray with a clock on the right.
fn draw_bottom_panel(canvas: &Canvas<'_>) {
    let palette = &canvas.palette;
    let (width, height) = (canvas.width, canvas.height);
    let panel = graphene::Rect::new(0.0, height * 0.89, width, height * 0.11);
    canvas.snapshot.append_color(&palette.bar, &panel);

    let icon_size = panel.height() * 0.6;
    let icon_top = panel.y() + (panel.height() - icon_size) / 2.0;
    let step = icon_size * 1.6;

    let launcher_x = panel.height() * 0.3;
    canvas.icon("lay-logo-kdeplasma-symbolic", launcher_x, icon_top, icon_size, &palette.accent);

    let tasks = ["utilities-terminal-symbolic", "folder-symbolic", "web-browser-symbolic"];
    let active_task = 2;
    for (index, icon_name) in tasks.iter().enumerate() {
        let x = launcher_x + step * (index as f32 + 1.4);
        canvas.icon(icon_name, x, icon_top, icon_size, &gdk::RGBA::WHITE);

        if index == active_task {
            let marker = graphene::Rect::new(x, panel.y() + panel.height() - 3.0, icon_size, 2.0);
            canvas.fill(marker, 1.0, &palette.accent);
        }
    }

    let tray = ["network-wireless-symbolic", "audio-volume-high-symbolic"];
    let clock_width = icon_size * 2.2;
    let clock_x = width - clock_width - panel.height() * 0.3;
    for (index, icon_name) in tray.iter().enumerate() {
        let x = clock_x - step * (tray.len() - index) as f32;
        canvas.icon(icon_name, x, icon_top, icon_size * 0.8, &gdk::RGBA::WHITE);
    }

    let clock_line = (panel.height() * 0.16).max(2.0);
    let clock_top = panel.y() + panel.height() / 2.0 - clock_line * 1.25;
    canvas.fill(graphene::Rect::new(clock_x, clock_top, clock_width, clock_line), clock_line / 2.0, &gdk::RGBA::WHITE);
    canvas.fill(
        graphene::Rect::new(clock_x + clock_width * 0.2, clock_top + clock_line * 1.5, clock_width * 0.6, clock_line),
        clock_line / 2.0,
        &palette.text,
    );
}

/// Draws the window with its app icon in the title bar. Returns the area below the title bar.
fn draw_floating_window(canvas: &Canvas<'_>, rect: graphene::Rect, icon_name: &str) -> graphene::Rect {
    let palette = &canvas.palette;
    let outline = gsk::RoundedRect::from_rect(rect, WINDOW_RADIUS);
    canvas
        .snapshot
        .append_outset_shadow(&outline, &WINDOW_SHADOW, 0.0, 6.0, 0.0, 18.0);

    let title_height = rect.height() * 0.16;
    canvas.snapshot.push_rounded_clip(&outline);
    canvas.snapshot.append_color(&palette.window, &rect);
    let title_bar = graphene::Rect::new(rect.x(), rect.y(), rect.width(), title_height);
    canvas.snapshot.append_color(&palette.title_bar, &title_bar);

    let icon_size = title_height * 0.6;
    let icon_margin = (title_height - icon_size) / 2.0;
    canvas.icon(icon_name, rect.x() + icon_margin * 2.0, rect.y() + icon_margin, icon_size, &ICON_ON_LIGHT);
    canvas.snapshot.pop();

    graphene::Rect::new(rect.x(), rect.y() + title_height, rect.width(), rect.height() - title_height)
}

/// An address bar, a large picture and two lines of text.
fn draw_browser_page(canvas: &Canvas<'_>, body: graphene::Rect) {
    let palette = &canvas.palette;
    let margin = body.width() * 0.07;
    let address = graphene::Rect::new(body.x() + body.width() * 0.2, body.y() + margin * 0.6, body.width() * 0.6, body.height() * 0.08);
    canvas.fill(address, address.height() / 2.0, &palette.title_bar);

    let picture_top = address.y() + address.height() + margin * 0.6;
    let picture = graphene::Rect::new(body.x() + margin, picture_top, body.width() - margin * 2.0, body.height() * 0.42);
    canvas.fill(picture, 6.0, &palette.glow);

    let line_height = (body.height() * 0.06).max(2.0);
    for (index, length) in [0.8, 0.55].iter().enumerate() {
        let y = picture.y() + picture.height() + margin * 0.6 + index as f32 * line_height * 2.0;
        let line = graphene::Rect::new(picture.x(), y, picture.width() * length, line_height);
        canvas.fill(line, line_height / 2.0, &palette.text);
    }
}

/// Album art with a music icon, a title and a progress bar.
fn draw_music_player(canvas: &Canvas<'_>, body: graphene::Rect) {
    let palette = &canvas.palette;
    let margin = body.width() * 0.08;
    let art_size = (body.height() - margin * 2.0).min(body.width() * 0.4);
    let art = graphene::Rect::new(body.x() + margin, body.y() + margin, art_size, art_size);
    canvas.fill(art, 6.0, &palette.accent);

    let note_size = art_size * 0.5;
    canvas.icon(
        "audio-x-generic-symbolic",
        art.x() + (art_size - note_size) / 2.0,
        art.y() + (art_size - note_size) / 2.0,
        note_size,
        &gdk::RGBA::WHITE,
    );

    let text_x = art.x() + art_size + margin;
    let text_width = body.x() + body.width() - margin - text_x;
    let line_height = (body.height() * 0.07).max(2.0);
    canvas.fill(graphene::Rect::new(text_x, art.y() + line_height, text_width * 0.8, line_height), line_height / 2.0, &palette.text);
    canvas.fill(graphene::Rect::new(text_x, art.y() + line_height * 3.0, text_width * 0.5, line_height), line_height / 2.0, &palette.text);

    let progress_y = art.y() + art_size - line_height;
    canvas.fill(graphene::Rect::new(text_x, progress_y, text_width, line_height * 0.8), line_height / 2.0, &palette.text);
    canvas.fill(graphene::Rect::new(text_x, progress_y, text_width * 0.45, line_height * 0.8), line_height / 2.0, &palette.accent);
}

/// A translucent dock with app icons on colored tiles.
fn draw_dock(canvas: &Canvas<'_>) {
    let palette = &canvas.palette;
    let dock = graphene::Rect::new(canvas.width * 0.3, canvas.height * 0.85, canvas.width * 0.4, canvas.height * 0.12);
    canvas.fill(dock, dock.height() * 0.35, &DOCK_COLOR);

    let tile_colors = [palette.accent, palette.bar, palette.glow, palette.accent];
    let tile = dock.height() * 0.72;
    let gap = (dock.width() - tile * DOCK_ICONS.len() as f32) / (DOCK_ICONS.len() as f32 + 1.0);
    let top = dock.y() + (dock.height() - tile) / 2.0;

    for (index, (icon_name, color)) in DOCK_ICONS.iter().zip(tile_colors.iter()).enumerate() {
        let x = dock.x() + gap + index as f32 * (tile + gap);
        canvas.fill(graphene::Rect::new(x, top, tile, tile), tile * 0.25, color);

        let glyph = tile * 0.6;
        canvas.icon(icon_name, x + (tile - glyph) / 2.0, top + (tile - glyph) / 2.0, glyph, &gdk::RGBA::WHITE);
    }
}

/// The text color at half strength, for calendar days that are not today.
fn dimmed(color: gdk::RGBA) -> gdk::RGBA {
    gdk::RGBA::new(color.red(), color.green(), color.blue(), color.alpha() * 0.45)
}
