//! Detects what the user runs. This module has no UI code.

mod os_release;
mod session;

use gettextrs::gettext;

pub use os_release::OsRelease;
pub use session::Session;

use crate::catalog::{Catalog, Distro};

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub os_release: OsRelease,
    pub session: Session,
}

impl SystemInfo {
    pub fn detect() -> Self {
        Self {
            os_release: OsRelease::load(),
            session: Session::detect(),
        }
    }

    pub fn is_current_distro(&self, distro: &Distro) -> bool {
        let same_id = self.os_release.id == distro.os_release_id;
        let same_variant = match &distro.os_release_variant {
            Some(variant) => self.os_release.variant_id.as_deref() == Some(variant.as_str()),
            None => true,
        };

        same_id && same_variant
    }

    /// Whether one of `session_names` is in the user's `XDG_CURRENT_DESKTOP`.
    pub fn is_running(&self, session_names: &[String]) -> bool {
        self.session.desktop_names.iter().any(|running| {
            session_names
                .iter()
                .any(|name| name.eq_ignore_ascii_case(running))
        })
    }

    pub fn distro_name(&self) -> String {
        if !self.os_release.name.is_empty() {
            return self.os_release.name.clone();
        }
        gettext("Linux")
    }

    /// The running desktop's name, preferring the catalog's spelling.
    pub fn desktop_name(&self, catalog: &Catalog) -> String {
        if let Some(desktop) = catalog.desktops.iter().find(|desktop| self.is_running(&desktop.session_names)) {
            return desktop.name.clone();
        }
        if let Some(compositor) = catalog.compositors.iter().find(|compositor| self.is_running(&compositor.session_names)) {
            return compositor.name.clone();
        }
        if let Some(name) = self.session.desktop_names.first() {
            return name.clone();
        }
        gettext("an unknown desktop")
    }
}
