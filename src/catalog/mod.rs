//! The catalog of distros, desktops and compositors.
//!
//! Today it is loaded from a snapshot bundled in the app's resources. Later it
//! is downloaded from the published catalog, with the snapshot as a fallback.

mod loader;
mod model;

pub use model::*;

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
