//! The catalog of distros, desktops and compositors.
//!
//! The app ships a snapshot in its resources, and `online` keeps a newer copy
//! downloaded from the catalog the Catalog workflow publishes.

mod features;
mod item_ref;
mod loader;
mod model;
mod online;
mod protocols;

pub use features::{key_features, FeatureSupport};
pub use item_ref::ItemRef;
pub use protocols::{ProtocolCategory, ProtocolInfo, ProtocolOverview};
pub use model::*;
pub use online::{download_now, refresh_downloaded, RefreshOutcome};

/// The few fields every catalog item has, used for small tiles and lists.
#[derive(Debug, Clone, Copy)]
pub struct ItemSummary<'a> {
    pub name: &'a str,
    pub short: &'a str,
    pub color: &'a str,
    pub logo: Option<&'a str>,
}

impl Catalog {
    pub fn find_distro(&self, id: &str) -> Option<&Distro> {
        self.distros.iter().find(|distro| distro.id == id)
    }

    pub fn find_desktop(&self, id: &str) -> Option<&Desktop> {
        self.desktops.iter().find(|desktop| desktop.id == id)
    }

    pub fn find_compositor(&self, id: &str) -> Option<&Compositor> {
        self.compositors.iter().find(|compositor| compositor.id == id)
    }

    /// The real screenshot of an item, if the catalog has one.
    pub fn hosted_screenshot(&self, kind: ItemKind, id: &str) -> Option<&HostedScreenshot> {
        match kind {
            ItemKind::Distro => self.find_distro(id)?.screenshot.as_ref(),
            ItemKind::Desktop => self.find_desktop(id)?.screenshot.as_ref(),
            ItemKind::Compositor => self.find_compositor(id)?.screenshot.as_ref(),
        }
    }

    /// The screenshots for an item's gallery: its gallery, or else its one
    /// real screenshot. Empty when it has neither.
    pub fn gallery(&self, kind: ItemKind, id: &str) -> Vec<HostedScreenshot> {
        let gallery = match kind {
            ItemKind::Distro => self.find_distro(id).map(|item| &item.gallery),
            ItemKind::Desktop => self.find_desktop(id).map(|item| &item.gallery),
            ItemKind::Compositor => self.find_compositor(id).map(|item| &item.gallery),
        };
        match gallery {
            Some(gallery) if !gallery.is_empty() => gallery.clone(),
            _ => self.hosted_screenshot(kind, id).into_iter().cloned().collect(),
        }
    }

    pub fn summary(&self, kind: ItemKind, id: &str) -> Option<ItemSummary<'_>> {
        match kind {
            ItemKind::Distro => self.find_distro(id).map(|item| ItemSummary {
                name: &item.name,
                short: &item.short,
                color: &item.color,
                logo: item.logo.as_deref(),
            }),
            ItemKind::Desktop => self.find_desktop(id).map(|item| ItemSummary {
                name: &item.name,
                short: &item.short,
                color: &item.color,
                logo: item.logo.as_deref(),
            }),
            ItemKind::Compositor => self.find_compositor(id).map(|item| ItemSummary {
                name: &item.name,
                short: &item.short,
                color: &item.color,
                logo: item.logo.as_deref(),
            }),
        }
    }
}
