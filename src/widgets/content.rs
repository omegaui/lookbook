//! Turns catalog items into the text, colors and logos a card shows.

use gettextrs::gettext;

use gtk::glib;

use super::{SceneLayout, TileContent};
use crate::catalog::{
    key_features, Accent, Catalog, Collection, Compositor, CompositorKind, Desktop, DesktopBar,
    DesktopLayout, Distro, FeatureSupport, HostedScreenshot, ItemKind, ItemRef, ItemSummary,
    Spotlight,
};
use crate::system::SystemInfo;

const EXTEND_ICON: &str = "application-x-addon-symbolic";
const DESKTOP_ICON: &str = "video-display-symbolic";
/// The bars drawn for a distro whose default desktop is not in the catalog.
const PLAIN_BARS: [DesktopBar; 2] = [DesktopBar::Top, DesktopBar::Dock];

/// A gallery card with art on top, for the Distros and Desktops pages.
#[derive(Debug, Clone)]
pub struct ShowcaseContent {
    pub card: CardContent,
    /// A short line under the name, for example "Wayland only" or "Arch-based".
    pub subtitle: String,
    pub footer_icon: &'static str,
    /// Pango markup, for example "Extend with <b>Shell extensions (GJS)</b>".
    pub footer_markup: String,
    /// The drawn placeholder's color and bars.
    pub color: String,
    pub bars: Vec<DesktopBar>,
    pub screenshot: Option<HostedScreenshot>,
    /// What the Desktops page's toolkit filter compares, for example "gtk".
    pub toolkit_id: Option<&'static str>,
}

impl ShowcaseContent {
    pub fn for_desktop(desktop: &Desktop, system: &SystemInfo) -> Self {
        Self {
            card: CardContent::for_desktop(desktop, system),
            subtitle: desktop.wayland.clone(),
            footer_icon: EXTEND_ICON,
            footer_markup: with_bold_value(&gettext("Extend with {}"), &desktop.extend_with),
            color: desktop.color.clone(),
            bars: desktop.bars.clone(),
            screenshot: desktop.screenshot.clone(),
            toolkit_id: Some(desktop.toolkit_family.id()),
        }
    }

    pub fn for_distro(distro: &Distro, catalog: &Catalog, system: &SystemInfo) -> Self {
        Self {
            card: CardContent::for_distro(distro, system),
            subtitle: distro.family.label(),
            footer_icon: DESKTOP_ICON,
            footer_markup: with_bold_value(&gettext("Desktop: {}"), &distro.default_desktop),
            color: distro.color.clone(),
            bars: default_desktop_bars(distro, catalog),
            screenshot: distro.screenshot.clone(),
            toolkit_id: None,
        }
    }
}

/// Fills `{}` in `template` with `value` in bold, escaped for Pango markup.
fn with_bold_value(template: &str, value: &str) -> String {
    let escaped_template = glib::markup_escape_text(template);
    let bold = format!("<b>{}</b>", glib::markup_escape_text(value));
    escaped_template.replace("{}", &bold)
}

/// The bars of the distro's default desktop, so the drawing looks like it.
fn default_desktop_bars(distro: &Distro, catalog: &Catalog) -> Vec<DesktopBar> {
    catalog
        .desktops
        .iter()
        .find(|desktop| desktop.name == distro.default_desktop)
        .map_or_else(|| PLAIN_BARS.to_vec(), |desktop| desktop.bars.clone())
}

/// Whether a compositor has one key feature, for example "Layer shell".
#[derive(Debug, Clone)]
pub struct FeatureCheck {
    pub label: String,
    pub supported: bool,
    /// How it is provided when that is not a Wayland protocol, for example
    /// "Through the ScreenCast portal".
    pub note: Option<String>,
}

fn feature_checks(compositor: &Compositor) -> Vec<FeatureCheck> {
    key_features()
        .into_iter()
        .map(|feature| {
            let support = feature.support_on(compositor);
            FeatureCheck {
                label: feature.label,
                supported: support != FeatureSupport::Missing,
                note: support_note(support),
            }
        })
        .collect()
}

fn support_note(support: FeatureSupport) -> Option<String> {
    match support {
        FeatureSupport::Portal(portal) => {
            Some(gettext("Through the {} portal").replace("{}", portal))
        }
        FeatureSupport::Protocol | FeatureSupport::Missing => None,
    }
}

/// The full protocol list of a compositor, for its details page.
#[derive(Debug, Clone)]
pub struct ProtocolSummary {
    pub version: String,
    pub checked_on: String,
    pub globals: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardBadge {
    YouAreHere,
    RunningNow,
}

impl CardBadge {
    pub fn label(self) -> String {
        match self {
            Self::YouAreHere => gettext("You are here"),
            Self::RunningNow => gettext("Running now"),
        }
    }

    pub fn css_class(self) -> &'static str {
        match self {
            Self::YouAreHere => "here",
            Self::RunningNow => "running",
        }
    }
}

fn tile_content(short: &str, color: &str, logo: Option<&str>) -> TileContent {
    TileContent {
        short: short.to_owned(),
        color: color.to_owned(),
        logo: logo.map(str::to_owned),
        icon_name: None,
    }
}

#[derive(Debug, Clone)]
pub struct CardContent {
    /// The item the card opens.
    pub item: ItemRef,
    pub tile: TileContent,
    pub name: String,
    pub description: String,
    pub chips: Vec<String>,
    pub badge: Option<CardBadge>,
    /// Key feature checks, shown on compositor cards only.
    pub checks: Vec<FeatureCheck>,
}

impl CardContent {
    pub fn for_distro(distro: &Distro, system: &SystemInfo) -> Self {
        Self {
            item: ItemRef::new(ItemKind::Distro, &distro.id),
            tile: tile_content(&distro.short, &distro.color, distro.logo.as_deref()),
            name: distro.name.clone(),
            description: distro.tagline.clone(),
            chips: vec![distro.release.label(), distro.package_manager.clone()],
            badge: system
                .is_current_distro(distro)
                .then_some(CardBadge::YouAreHere),
            checks: Vec::new(),
        }
    }

    pub fn for_desktop(desktop: &Desktop, system: &SystemInfo) -> Self {
        Self {
            item: ItemRef::new(ItemKind::Desktop, &desktop.id),
            tile: tile_content(&desktop.short, &desktop.color, desktop.logo.as_deref()),
            name: desktop.name.clone(),
            description: desktop.tagline.clone(),
            chips: vec![desktop.compositor.clone(), desktop.toolkit.clone()],
            badge: system
                .runs_desktop(&desktop.id)
                .then_some(CardBadge::RunningNow),
            checks: Vec::new(),
        }
    }

    pub fn for_compositor(compositor: &Compositor, system: &SystemInfo) -> Self {
        Self {
            item: ItemRef::new(ItemKind::Compositor, &compositor.id),
            tile: tile_content(&compositor.short, &compositor.color, compositor.logo.as_deref()),
            name: compositor.name.clone(),
            description: compositor.tagline.clone(),
            chips: vec![compositor.kind.label(), compositor.base.clone()],
            badge: system
                .runs_compositor(&compositor.id)
                .then_some(CardBadge::RunningNow),
            checks: feature_checks(compositor),
        }
    }
}

/// One labelled value, for example "Packages: dnf", with an optional second line.
#[derive(Debug, Clone)]
pub struct Fact {
    pub label: String,
    pub value: String,
    pub detail: Option<String>,
    /// A colored tile with a logo or icon, for the Your System facts.
    pub tile: Option<TileContent>,
}

impl Fact {
    pub fn new(label: String, value: &str) -> Self {
        Self {
            label,
            value: value.to_owned(),
            tile: None,
            detail: None,
        }
    }

    pub fn with_detail(label: String, value: &str, detail: &str) -> Self {
        Self {
            detail: Some(detail.to_owned()),
            ..Self::new(label, value)
        }
    }

    pub fn tile(self, tile: TileContent) -> Self {
        Self { tile: Some(tile), ..self }
    }
}

/// A tile with a symbolic icon on a fixed color, for facts with no logo.
pub fn icon_tile(icon_name: &str, color: &str) -> TileContent {
    TileContent {
        short: String::new(),
        color: color.to_owned(),
        logo: None,
        icon_name: Some(icon_name.to_owned()),
    }
}

/// A tile with an item's logo on its brand color.
pub fn logo_tile(short: &str, color: &str, logo: Option<&str>) -> TileContent {
    tile_content(short, color, logo)
}

fn fact(label: String, value: &str) -> Fact {
    Fact::new(label, value)
}

/// Everything the details page shows about one item.
#[derive(Debug, Clone)]
pub struct DetailContent {
    pub item: ItemRef,
    pub title: String,
    pub tile: TileContent,
    pub tagline: String,
    pub badge: Option<CardBadge>,
    /// Short facts under the tagline, for example "Rolling" and "pacman".
    pub chips: Vec<String>,
    /// "At a Glance", or "Inside GNOME" for desktops.
    pub facts_title: String,
    pub facts: Vec<Fact>,
    pub website: String,
    /// The brand color the header scene is drawn in.
    pub color: String,
    pub scene: SceneLayout,
    /// Compositors only.
    pub checks: Vec<FeatureCheck>,
    /// Compositors only.
    pub protocols: Option<ProtocolSummary>,
}

impl DetailContent {
    pub fn from_catalog(item: &ItemRef, catalog: &Catalog, system: &SystemInfo) -> Option<Self> {
        match item.kind {
            ItemKind::Distro => catalog
                .find_distro(&item.id)
                .map(|distro| Self::for_distro(distro, system)),
            ItemKind::Desktop => catalog
                .find_desktop(&item.id)
                .map(|desktop| Self::for_desktop(desktop, system)),
            ItemKind::Compositor => catalog
                .find_compositor(&item.id)
                .map(|compositor| Self::for_compositor(compositor, system)),
        }
    }

    fn for_distro(distro: &Distro, system: &SystemInfo) -> Self {
        Self {
            item: ItemRef::new(ItemKind::Distro, &distro.id),
            title: distro.name.clone(),
            tile: tile_content(&distro.short, &distro.color, distro.logo.as_deref()),
            tagline: distro.tagline.clone(),
            badge: system
                .is_current_distro(distro)
                .then_some(CardBadge::YouAreHere),
            chips: vec![distro.release.label(), distro.package_manager.clone()],
            facts_title: gettext("At a Glance"),
            facts: vec![
                fact(gettext("Based on"), &distro.family.label()),
                fact(gettext("Releases"), &distro.release.label()),
                fact(gettext("Packages"), &distro.package_manager),
                fact(gettext("Desktop"), &distro.default_desktop),
            ],
            website: distro.website.clone(),
            color: distro.color.clone(),
            scene: SceneLayout::Floating,
            checks: Vec::new(),
            protocols: None,
        }
    }

    fn for_desktop(desktop: &Desktop, system: &SystemInfo) -> Self {
        Self {
            item: ItemRef::new(ItemKind::Desktop, &desktop.id),
            title: desktop.name.clone(),
            tile: tile_content(&desktop.short, &desktop.color, desktop.logo.as_deref()),
            tagline: desktop.tagline.clone(),
            badge: system
                .runs_desktop(&desktop.id)
                .then_some(CardBadge::RunningNow),
            chips: vec![desktop.wayland.clone(), desktop.toolkit.clone()],
            facts_title: gettext("Inside {}").replace("{}", &desktop.name),
            facts: vec![
                fact(gettext("Compositor"), &desktop.compositor),
                fact(gettext("Shell"), &desktop.shell),
                fact(gettext("Toolkit"), &desktop.toolkit),
                fact(gettext("Extend with"), &desktop.extend_with),
            ],
            website: desktop.website.clone(),
            color: desktop.color.clone(),
            scene: scene_for_desktop(desktop.layout),
            checks: Vec::new(),
            protocols: None,
        }
    }

    fn for_compositor(compositor: &Compositor, system: &SystemInfo) -> Self {
        Self {
            item: ItemRef::new(ItemKind::Compositor, &compositor.id),
            title: compositor.name.clone(),
            tile: tile_content(&compositor.short, &compositor.color, compositor.logo.as_deref()),
            tagline: compositor.tagline.clone(),
            badge: system
                .runs_compositor(&compositor.id)
                .then_some(CardBadge::RunningNow),
            chips: vec![compositor.kind.label(), compositor.base.clone()],
            facts_title: gettext("At a Glance"),
            facts: vec![
                fact(gettext("Layout"), &compositor.kind.label()),
                fact(gettext("Built on"), &compositor.base),
                fact(gettext("Config"), &compositor.config),
            ],
            website: compositor.website.clone(),
            color: compositor.color.clone(),
            scene: scene_for_compositor(compositor.kind),
            checks: feature_checks(compositor),
            protocols: Some(ProtocolSummary {
                version: compositor.protocols.version.clone(),
                checked_on: compositor.protocols.checked_on.clone(),
                globals: compositor.protocols.globals.clone(),
            }),
        }
    }
}

pub fn scene_for_desktop(layout: DesktopLayout) -> SceneLayout {
    match layout {
        DesktopLayout::Dock => SceneLayout::Floating,
        DesktopLayout::Panel => SceneLayout::Panel,
    }
}

#[derive(Debug, Clone)]
pub struct CollectionContent {
    pub title: String,
    pub names: String,
    pub tiles: Vec<TileContent>,
    pub accent: Accent,
}

impl CollectionContent {
    pub fn from_catalog(collection: &Collection, catalog: &Catalog) -> Self {
        let members: Vec<ItemSummary<'_>> = collection
            .members
            .iter()
            .filter_map(|id| catalog.summary(collection.kind, id))
            .collect();

        let names: Vec<&str> = members.iter().map(|member| member.name).collect();
        let tiles = members
            .iter()
            .map(|member| tile_content(member.short, member.color, member.logo))
            .collect();

        Self {
            title: collection.title.clone(),
            names: names.join(" · "),
            tiles,
            accent: collection.accent,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpotlightContent {
    pub kicker: String,
    pub tile: TileContent,
    pub name: String,
    pub tagline: String,
    pub chips: Vec<String>,
    pub accent: Accent,
    /// How the drawn scene arranges its windows.
    pub scene: SceneLayout,
    /// The item "View Details" and "Can I Run It?" are about.
    pub item: ItemRef,
    pub screenshot: Option<HostedScreenshot>,
}

/// Compositors show their own window layout; desktops and distros float windows.
fn scene_for_compositor(kind: CompositorKind) -> SceneLayout {
    match kind {
        CompositorKind::Scrolling => SceneLayout::Scrolling,
        CompositorKind::Tiling => SceneLayout::Tiling,
        CompositorKind::Stacking => SceneLayout::Floating,
    }
}

impl SpotlightContent {
    pub fn from_catalog(spotlight: &Spotlight, catalog: &Catalog) -> Option<Self> {
        match spotlight.kind {
            ItemKind::Distro => catalog
                .find_distro(&spotlight.id)
                .map(|distro| Self::for_distro(spotlight, distro)),
            ItemKind::Desktop => catalog
                .find_desktop(&spotlight.id)
                .map(|desktop| Self::for_desktop(spotlight, desktop)),
            ItemKind::Compositor => catalog
                .find_compositor(&spotlight.id)
                .map(|compositor| Self::for_compositor(spotlight, compositor)),
        }
    }

    fn for_distro(spotlight: &Spotlight, distro: &Distro) -> Self {
        Self {
            kicker: gettext("Distro spotlight"),
            tile: tile_content(&distro.short, &distro.color, distro.logo.as_deref()),
            name: distro.name.clone(),
            tagline: spotlight.tagline.clone(),
            chips: vec![distro.release.label(), distro.package_manager.clone()],
            accent: spotlight.accent,
            scene: SceneLayout::Floating,
            item: ItemRef::new(ItemKind::Distro, &distro.id),
            screenshot: distro.screenshot.clone(),
        }
    }

    fn for_desktop(spotlight: &Spotlight, desktop: &Desktop) -> Self {
        Self {
            kicker: gettext("Desktop spotlight"),
            tile: tile_content(&desktop.short, &desktop.color, desktop.logo.as_deref()),
            name: desktop.name.clone(),
            tagline: spotlight.tagline.clone(),
            chips: vec![desktop.compositor.clone(), desktop.toolkit.clone()],
            accent: spotlight.accent,
            scene: scene_for_desktop(desktop.layout),
            item: ItemRef::new(ItemKind::Desktop, &desktop.id),
            screenshot: desktop.screenshot.clone(),
        }
    }

    fn for_compositor(spotlight: &Spotlight, compositor: &Compositor) -> Self {
        Self {
            kicker: gettext("Compositor spotlight"),
            tile: tile_content(&compositor.short, &compositor.color, compositor.logo.as_deref()),
            name: compositor.name.clone(),
            tagline: spotlight.tagline.clone(),
            chips: vec![
                compositor.kind.label(),
                compositor.base.clone(),
                compositor.config.clone(),
            ],
            accent: spotlight.accent,
            scene: scene_for_compositor(compositor.kind),
            item: ItemRef::new(ItemKind::Compositor, &compositor.id),
            screenshot: compositor.screenshot.clone(),
        }
    }
}
