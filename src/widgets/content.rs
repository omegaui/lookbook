//! Turns catalog items into the text, colors and logos a card shows.

use gettextrs::gettext;

use super::TileContent;
use crate::catalog::{
    Accent, Catalog, Collection, Compositor, Desktop, Distro, ItemKind, ItemSummary, Spotlight,
};
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
    pub tile: TileContent,
    pub name: String,
    pub description: String,
    pub chips: Vec<String>,
    pub badge: Option<CardBadge>,
}

impl CardContent {
    pub fn for_distro(distro: &Distro, system: &SystemInfo) -> Self {
        Self {
            tile: tile_content(&distro.short, &distro.color, distro.logo.as_deref()),
            name: distro.name.clone(),
            description: distro.tagline.clone(),
            chips: vec![distro.release.label(), distro.package_manager.clone()],
            badge: system
                .is_current_distro(distro)
                .then_some(CardBadge::YouAreHere),
        }
    }

    pub fn for_desktop(desktop: &Desktop, system: &SystemInfo) -> Self {
        Self {
            tile: tile_content(&desktop.short, &desktop.color, desktop.logo.as_deref()),
            name: desktop.name.clone(),
            description: desktop.tagline.clone(),
            chips: vec![desktop.compositor.clone(), desktop.toolkit.clone()],
            badge: system
                .is_running(&desktop.session_names)
                .then_some(CardBadge::RunningNow),
        }
    }

    pub fn for_compositor(compositor: &Compositor, system: &SystemInfo) -> Self {
        Self {
            tile: tile_content(&compositor.short, &compositor.color, compositor.logo.as_deref()),
            name: compositor.name.clone(),
            description: compositor.tagline.clone(),
            chips: vec![compositor.kind.label(), compositor.base.clone()],
            badge: system
                .is_running(&compositor.session_names)
                .then_some(CardBadge::RunningNow),
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
