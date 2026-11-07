use gettextrs::gettext;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Catalog {
    pub schema_version: u32,
    pub generated_at: String,
    pub collections: Vec<Collection>,
    pub spotlights: Vec<Spotlight>,
    pub screenshots: Vec<Screenshot>,
    pub distros: Vec<Distro>,
    pub desktops: Vec<Desktop>,
    pub compositors: Vec<Compositor>,
    pub ideas: Vec<Idea>,
}

/// Something people build on their desktop, like a status bar or a lock screen.
#[derive(Debug, Clone, Deserialize)]
pub struct Idea {
    pub id: String,
    pub name: String,
    pub description: String,
    /// The ways to build it. The idea is ready when any one of them works on the user's system.
    pub routes: Vec<IdeaRoute>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RouteKind {
    /// Works on compositors that offer every global in `requires`.
    Protocol,
    /// Works on the desktops in `desktops`, for example as a GNOME Shell extension.
    Desktop,
    /// Works through an XDG desktop portal, on any desktop.
    Portal,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IdeaRoute {
    pub kind: RouteKind,
    /// How to build it this way, for example "Layer shell" or "Shell extension (GJS)".
    pub label: String,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub desktops: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Distro,
    Desktop,
    Compositor,
}

impl ItemKind {
    pub fn id(self) -> &'static str {
        match self {
            Self::Distro => "distro",
            Self::Desktop => "desktop",
            Self::Compositor => "compositor",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "distro" => Some(Self::Distro),
            "desktop" => Some(Self::Desktop),
            "compositor" => Some(Self::Compositor),
            _ => None,
        }
    }
}

/// Colors from the GNOME palette, used for collections and spotlights.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Accent {
    Blue,
    Green,
    Yellow,
    Orange,
    Red,
    Purple,
    Brown,
    Dark,
}

impl Accent {
    pub fn css_class(self) -> &'static str {
        match self {
            Self::Blue => "accent-blue",
            Self::Green => "accent-green",
            Self::Yellow => "accent-yellow",
            Self::Orange => "accent-orange",
            Self::Red => "accent-red",
            Self::Purple => "accent-purple",
            Self::Brown => "accent-brown",
            Self::Dark => "accent-dark",
        }
    }
}

/// A themed group of items, shown on the tilted cards at the top of Explore.
#[derive(Debug, Clone, Deserialize)]
pub struct Collection {
    pub title: String,
    pub kind: ItemKind,
    pub members: Vec<String>,
    pub accent: Accent,
}

/// An item featured in the Explore carousel.
#[derive(Debug, Clone, Deserialize)]
pub struct Spotlight {
    pub kind: ItemKind,
    pub id: String,
    pub tagline: String,
    pub accent: Accent,
}

/// A Wall of fame entry. Until real screenshots are in the catalog, each one
/// carries the colors for a drawn mock-up.
#[derive(Debug, Clone, Deserialize)]
pub struct Screenshot {
    pub title: String,
    pub author: String,
    pub subject: String,
    pub mock: MockLook,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MockLayout {
    Tiled,
    Floating,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MockLook {
    pub layout: MockLayout,
    pub wallpaper: String,
    pub glow: String,
    pub bar: String,
    pub window: String,
    pub title_bar: String,
    pub accent: String,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DistroFamily {
    Independent,
    Arch,
    Debian,
    Fedora,
}

impl DistroFamily {
    /// Matches the toggle names in the family filters.
    pub fn id(self) -> &'static str {
        match self {
            Self::Independent => "independent",
            Self::Arch => "arch",
            Self::Debian => "debian",
            Self::Fedora => "fedora",
        }
    }

    pub fn label(self) -> String {
        match self {
            Self::Independent => gettext("Independent"),
            Self::Arch => gettext("Arch-based"),
            Self::Debian => gettext("Debian-based"),
            Self::Fedora => gettext("Fedora-based"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseModel {
    Rolling,
    Fixed,
    Atomic,
}

impl ReleaseModel {
    /// Matches the toggle names in the Distros page's release filter.
    pub fn id(self) -> &'static str {
        match self {
            Self::Rolling => "rolling",
            Self::Fixed => "fixed",
            Self::Atomic => "atomic",
        }
    }

    pub fn label(self) -> String {
        match self {
            Self::Rolling => gettext("Rolling"),
            Self::Fixed => gettext("Fixed"),
            Self::Atomic => gettext("Atomic"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Distro {
    pub id: String,
    pub name: String,
    pub short: String,
    pub color: String,
    pub tagline: String,
    pub family: DistroFamily,
    pub release: ReleaseModel,
    pub package_manager: String,
    pub website: String,
    /// A bundled logo id, for example "fedora". Items without one show `short`.
    pub logo: Option<String>,
    /// The `ID` field of this distro's os-release file.
    pub os_release_id: String,
    /// The `VARIANT_ID` field, for distros that share an `ID`, like Fedora editions.
    pub os_release_variant: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Desktop {
    pub id: String,
    pub name: String,
    pub short: String,
    pub color: String,
    pub tagline: String,
    pub compositor: String,
    pub toolkit: String,
    pub website: String,
    pub logo: Option<String>,
    /// Values of `XDG_CURRENT_DESKTOP` that mean this desktop is running.
    pub session_names: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompositorKind {
    Tiling,
    Scrolling,
    Stacking,
}

impl CompositorKind {
    /// Matches the toggle names in the Compositors page's layout filter.
    pub fn id(self) -> &'static str {
        match self {
            Self::Tiling => "tiling",
            Self::Scrolling => "scrolling",
            Self::Stacking => "stacking",
        }
    }

    pub fn label(self) -> String {
        match self {
            Self::Tiling => gettext("Tiling"),
            Self::Scrolling => gettext("Scrolling"),
            Self::Stacking => gettext("Stacking"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Compositor {
    pub id: String,
    pub name: String,
    pub short: String,
    pub color: String,
    pub tagline: String,
    pub kind: CompositorKind,
    pub base: String,
    pub config: String,
    pub website: String,
    pub logo: Option<String>,
    /// Values of `XDG_CURRENT_DESKTOP` that mean this compositor is running.
    pub session_names: Vec<String>,
    /// Starts of global names that only this compositor offers, for example "hyprland_".
    pub fingerprints: Vec<String>,
    pub protocols: ProtocolSupport,
}

impl Compositor {
    pub fn supports(&self, interface: &str) -> bool {
        self.protocols.globals.iter().any(|global| global == interface)
    }

    /// Whether any global the compositor offers starts with one of its fingerprints.
    pub fn matches_fingerprint(&self, globals: &[String]) -> bool {
        self.fingerprints.iter().any(|fingerprint| {
            globals.iter().any(|global| global.starts_with(fingerprint.as_str()))
        })
    }
}

/// The Wayland globals a compositor offers, from wayland-explorer
/// (https://github.com/vially/wayland-explorer).
#[derive(Debug, Clone, Deserialize)]
pub struct ProtocolSupport {
    /// The compositor version the list was taken from.
    pub version: String,
    pub checked_on: String,
    pub globals: Vec<String>,
}
