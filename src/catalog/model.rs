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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Distro,
    Desktop,
    Compositor,
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
    /// Matches the toggle names in the Explore page's family filter.
    pub fn id(self) -> &'static str {
        match self {
            Self::Independent => "independent",
            Self::Arch => "arch",
            Self::Debian => "debian",
            Self::Fedora => "fedora",
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
    pub logo: Option<String>,
    /// Values of `XDG_CURRENT_DESKTOP` that mean this compositor is running.
    pub session_names: Vec<String>,
}
