//! Turns catalog items into the text, colors and logos a card shows.

use gettextrs::gettext;

use super::TileContent;
use crate::catalog::{
    key_features, Accent, Catalog, Collection, Compositor, Desktop, Distro, ItemKind, ItemRef,
    ItemSummary, Spotlight,
};

/// Whether a compositor has one key feature, for example "Layer shell".
#[derive(Debug, Clone)]
pub struct FeatureCheck {
    pub label: String,
    pub supported: bool,
}

fn feature_checks(compositor: &Compositor) -> Vec<FeatureCheck> {
    key_features()
        .into_iter()
        .map(|feature| FeatureCheck {
            supported: feature.is_supported_by(compositor),
            label: feature.label,
        })
        .collect()
}

/// The full protocol list of a compositor, for its details page.
#[derive(Debug, Clone)]
pub struct ProtocolSummary {
    pub version: String,
    pub checked_on: String,
    pub globals: Vec<String>,
}
use crate::system::SystemInfo;

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
                .is_running(&desktop.session_names)
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
                .is_running(&compositor.session_names)
                .then_some(CardBadge::RunningNow),
            checks: feature_checks(compositor),
        }
    }
}

/// One labelled value on a details page, for example "Packages: dnf".
#[derive(Debug, Clone)]
pub struct Fact {
    pub label: String,
    pub value: String,
}

fn fact(label: String, value: &str) -> Fact {
    Fact {
        label,
        value: value.to_owned(),
    }
}

/// Everything the details page shows about one item.
#[derive(Debug, Clone)]
pub struct DetailContent {
    pub title: String,
    pub tile: TileContent,
    pub tagline: String,
    pub badge: Option<CardBadge>,
    pub facts: Vec<Fact>,
    pub website: String,
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
            title: distro.name.clone(),
            tile: tile_content(&distro.short, &distro.color, distro.logo.as_deref()),
            tagline: distro.tagline.clone(),
            badge: system
                .is_current_distro(distro)
                .then_some(CardBadge::YouAreHere),
            facts: vec![
                fact(gettext("Based on"), &distro.family.label()),
                fact(gettext("Releases"), &distro.release.label()),
                fact(gettext("Packages"), &distro.package_manager),
            ],
            website: distro.website.clone(),
            checks: Vec::new(),
            protocols: None,
        }
    }

    fn for_desktop(desktop: &Desktop, system: &SystemInfo) -> Self {
        Self {
            title: desktop.name.clone(),
            tile: tile_content(&desktop.short, &desktop.color, desktop.logo.as_deref()),
            tagline: desktop.tagline.clone(),
            badge: system
                .is_running(&desktop.session_names)
                .then_some(CardBadge::RunningNow),
            facts: vec![
                fact(gettext("Compositor"), &desktop.compositor),
                fact(gettext("Toolkit"), &desktop.toolkit),
            ],
            website: desktop.website.clone(),
            checks: Vec::new(),
            protocols: None,
        }
    }

    fn for_compositor(compositor: &Compositor, system: &SystemInfo) -> Self {
        Self {
            title: compositor.name.clone(),
            tile: tile_content(&compositor.short, &compositor.color, compositor.logo.as_deref()),
            tagline: compositor.tagline.clone(),
            badge: system
                .is_running(&compositor.session_names)
                .then_some(CardBadge::RunningNow),
            facts: vec![
                fact(gettext("Layout"), &compositor.kind.label()),
                fact(gettext("Built on"), &compositor.base),
                fact(gettext("Config"), &compositor.config),
            ],
            website: compositor.website.clone(),
            checks: feature_checks(compositor),
            protocols: Some(ProtocolSummary {
                version: compositor.protocols.version.clone(),
                checked_on: compositor.protocols.checked_on.clone(),
                globals: compositor.protocols.globals.clone(),
            }),
        }
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
    /// The view stack page that "View details" opens.
    pub page_name: &'static str,
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
            page_name: "distros",
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
            page_name: "desktops",
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
            page_name: "compositors",
        }
    }
}
