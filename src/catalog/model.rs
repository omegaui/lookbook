use gettextrs::gettext;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Catalog {
    pub schema_version: u32,
    pub generated_at: String,
    pub collections: Vec<Collection>,
    pub spotlights: Vec<Spotlight>,
    /// The last spotlight: a wallpaper picked live from one source.
    #[serde(default)]
    pub wallpaper_spotlight: Option<WallpaperSpotlight>,
    /// Winners of Hyprland's ricing contests, newest contest first.
    #[serde(default)]
    pub hall_of_fame: HallOfFame,
    pub distros: Vec<Distro>,
    pub desktops: Vec<Desktop>,
    pub compositors: Vec<Compositor>,
    pub ideas: Vec<Idea>,
    /// Plain-language descriptions of the protocols that matter to builders.
    #[serde(default)]
    pub protocols: Vec<super::ProtocolInfo>,
    #[serde(default)]
    pub wallpaper_sources: Vec<WallpaperSource>,
    /// Image URLs hidden on request, for example after a takedown request.
    #[serde(default)]
    pub wallpaper_blocklist: Vec<String>,
    /// Where "What's new this month" reads recent releases.
    #[serde(default)]
    pub news_sources: Vec<NewsSource>,
    /// Complete setups people share for a compositor, like end-4's dots.
    #[serde(default)]
    pub rices: Vec<Rice>,
}

/// A rice: someone's complete, shareable setup of bars, launchers, themes and
/// more, installed from their repository.
#[derive(Debug, Clone, Deserialize)]
pub struct Rice {
    pub id: String,
    pub name: String,
    /// Who makes it, for example "end-4".
    pub author: String,
    pub tagline: String,
    /// The compositors (by id) it runs on.
    pub compositors: Vec<String>,
    /// What its shell is made with, for example "Quickshell" or "Waybar".
    pub built_with: Vec<String>,
    /// The distros its installer supports, for example "Arch, Fedora".
    pub installs_on: Option<String>,
    /// Its repository, where the install guide lives.
    pub link: String,
    pub license: String,
    /// A picture or clip from its own README, loaded live and never bundled.
    pub preview: RicePreview,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RicePreview {
    pub kind: RicePreviewKind,
    pub url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RicePreviewKind {
    Image,
    Video,
}

impl Rice {
    pub fn runs_on(&self, compositor_id: &str) -> bool {
        self.compositors.iter().any(|id| id == compositor_id)
    }
}

/// One project's release feed, for "What's new this month".
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct NewsSource {
    pub id: String,
    /// For example "Hyprland".
    pub name: String,
    pub category: NewsCategory,
    pub kind: NewsSourceKind,
    /// The feed itself.
    pub url: String,
    /// Where a release card leads when the feed has no link of its own.
    pub link: String,
    /// A bundled logo id, for example "linux".
    pub logo: Option<String>,
    /// Letters for the tile when there is no logo.
    pub short: String,
    pub color: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NewsCategory {
    Kernel,
    Distro,
    Desktop,
    Compositor,
}

/// How to read a feed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NewsSourceKind {
    /// kernel.org's `releases.json`.
    KernelOrg,
    /// The endoflife.date API, version 1.
    Endoflife,
    /// An Atom feed, like GitHub's or GitLab's release and tag feeds.
    Atom,
    /// An RSS 2.0 feed, like Codeberg's.
    Rss,
}

/// Where a wallpaper collection lives, and how to read its live list.
#[derive(Debug, Clone, Deserialize)]
pub struct WallpaperSource {
    pub id: String,
    pub name: String,
    pub kind: WallpaperSourceKind,
    /// The address the live list is read from.
    pub index_url: String,
    /// The page people visit to see the collection.
    pub link: String,
    pub credit: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WallpaperSourceKind {
    /// A copyparty file server, which lists files as JSON and makes thumbnails.
    Copyparty,
    /// A WordPress page whose images carry a `srcset` of sizes.
    WordpressPage,
}

/// Something people build on their desktop, like a status bar or a lock screen.
#[derive(Debug, Clone, Deserialize)]
pub struct Idea {
    pub id: String,
    pub name: String,
    pub description: String,
    /// A symbolic icon name, from the icon theme or the app's own icons.
    pub icon: String,
    /// The tile color for the idea's category, for example blue for shell tools.
    pub color: String,
    /// Desktops and compositors (by id) that already include it, so nobody
    /// needs to build it there. For example, GNOME has its own lock screen.
    #[serde(default)]
    pub built_into: Vec<String>,
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
    /// For portal routes: the portal it needs, for example "Screenshot".
    #[serde(default)]
    pub portal: Option<String>,
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

/// A themed group of items, shown on the cards at the top of Explore.
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

/// The wallpaper spotlight: which source to pick from, and what to say about it.
#[derive(Debug, Clone, Deserialize)]
pub struct WallpaperSpotlight {
    pub source_id: String,
    pub tagline: String,
    pub accent: Accent,
}

/// The Hall of fame: winners of Hyprland's ricing contests, read from the
/// Hyprland website. Their screenshots stay on hypr.land.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct HallOfFame {
    /// Hyprland's Hall of Fame page, with every contest.
    pub page: String,
    pub contests: Vec<Contest>,
}

/// One ricing contest, for example "Moon" in June 2026.
#[derive(Debug, Clone, Deserialize)]
pub struct Contest {
    pub name: String,
    pub number: u32,
    pub date: String,
    pub rices: Vec<FamedRice>,
}

/// A winning rice: its screenshot, author, place and dotfiles.
#[derive(Debug, Clone, Deserialize)]
pub struct FamedRice {
    pub name: String,
    pub author: String,
    /// For example "#1" or "#2 (ex aequo)".
    pub place: String,
    pub image: String,
    /// The rice's dotfiles, or the Hall of Fame page when they were never shared.
    pub link: String,
}

impl HallOfFame {
    /// The first `count` winners, newest contest first and best place first,
    /// each with its contest.
    pub fn newest(&self, count: usize) -> Vec<(&Contest, &FamedRice)> {
        self.contests
            .iter()
            .flat_map(|contest| contest.rices.iter().map(move |rice| (contest, rice)))
            .take(count)
            .collect()
    }
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
    /// For example "GNOME", or "Your choice" when the installer asks.
    pub default_desktop: String,
    /// The catalog id of that desktop, when it is one desktop the catalog has.
    #[serde(default)]
    pub default_desktop_id: Option<String>,
    /// Where Repology lists this distro's packages, in the order to look.
    #[serde(default)]
    pub repology_repos: Vec<RepologyRepo>,
    /// What NVIDIA users should know, for example where to get the driver.
    pub nvidia_note: Option<String>,
    /// A real screenshot, loaded live from where it is hosted.
    pub screenshot: Option<HostedScreenshot>,
    /// Screenshots for the details page's gallery, best first.
    #[serde(default)]
    pub gallery: Vec<HostedScreenshot>,
}

/// One of a distro's package repositories as Repology names it.
#[derive(Debug, Clone, Deserialize)]
pub struct RepologyRepo {
    /// Repology's name for it. `{version}` stands for os-release `VERSION_ID`
    /// with dots turned into underscores, for example "fedora_{version}".
    pub repo: String,
    /// A short name shown next to the distro's, for example "AUR".
    pub label: Option<String>,
    /// How to install from it. `{package}` is the package name and
    /// `{codename}` is os-release `VERSION_CODENAME`.
    pub install: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Desktop {
    pub id: String,
    pub name: String,
    pub short: String,
    pub color: String,
    pub tagline: String,
    /// The compositor's name as people know it, for example "Mutter".
    pub compositor: String,
    /// Its catalog id, when the catalog has it, for example "mutter".
    #[serde(default)]
    pub compositor_id: Option<String>,
    pub toolkit: String,
    pub website: String,
    pub logo: Option<String>,
    /// Values of `XDG_CURRENT_DESKTOP` that mean this desktop is running.
    pub session_names: Vec<String>,
    /// The program that draws panels and menus, for example "GNOME Shell".
    pub shell: String,
    /// How people extend it, for example "Shell extensions (GJS)".
    pub extend_with: String,
    /// How far along its Wayland session is, for example "Wayland only".
    pub wayland: String,
    pub toolkit_family: ToolkitFamily,
    pub layout: DesktopLayout,
    /// The bars it shows by default, drawn on the gallery card.
    #[serde(default)]
    pub bars: Vec<DesktopBar>,
    /// Repology's project name for its main package, for example "gnome-shell".
    pub repology: Option<String>,
    /// A real screenshot, loaded live from where it is hosted.
    pub screenshot: Option<HostedScreenshot>,
    /// Screenshots for the details page's gallery, best first.
    #[serde(default)]
    pub gallery: Vec<HostedScreenshot>,
}

/// A screenshot the app shows from its host, never bundled, with its credit.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct HostedScreenshot {
    /// The image itself, for example a 960px Wikimedia Commons thumbnail.
    pub url: String,
    /// The page that describes the image and its license.
    pub page: String,
    pub credit: String,
    pub license: String,
    /// What the screenshot shows, for example "Activities overview".
    #[serde(default)]
    pub caption: Option<String>,
}

/// A bar a desktop shows by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DesktopBar {
    /// A thin bar across the top, like GNOME's.
    Top,
    /// A full-width panel along the bottom, like KDE Plasma's taskbar.
    Bottom,
    /// A floating row of app icons near the bottom.
    Dock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolkitFamily {
    Gtk,
    Qt,
    Rust,
}

impl ToolkitFamily {
    /// Matches the toggle names in the Desktops page's toolkit filter.
    pub fn id(self) -> &'static str {
        match self {
            Self::Gtk => "gtk",
            Self::Qt => "qt",
            Self::Rust => "rust",
        }
    }
}

/// Where the desktop's panel sits by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DesktopLayout {
    /// A top bar and a dock, like GNOME.
    Dock,
    /// A full-width bottom panel, like KDE Plasma.
    Panel,
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
    /// XDG desktop portals its usual portal backend provides, for example "ScreenCast".
    pub portals: Vec<String>,
    /// Repology's project name, for example "hyprland".
    pub repology: Option<String>,
    /// What NVIDIA users should know before trying it.
    pub nvidia_note: Option<String>,
    /// A real screenshot, loaded live from where it is hosted.
    pub screenshot: Option<HostedScreenshot>,
    /// Screenshots for the details page's gallery, best first.
    #[serde(default)]
    pub gallery: Vec<HostedScreenshot>,
}

impl Compositor {
    pub fn has_portal(&self, portal: &str) -> bool {
        self.portals.iter().any(|name| name == portal)
    }

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
