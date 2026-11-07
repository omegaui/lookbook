//! Detects what the user runs. This module has no UI code.

mod gpu;
mod hardware;
mod os_release;
mod session;
mod wayland;

use gettextrs::gettext;

pub use hardware::Hardware;
pub use os_release::OsRelease;
pub use session::Session;

use crate::catalog::{Catalog, Compositor, Desktop, Distro};

#[derive(Debug, Clone, Default)]
pub struct SystemInfo {
    pub os_release: OsRelease,
    pub session: Session,
    pub hardware: Hardware,
    /// The Wayland globals visible from inside the sandbox.
    pub visible_globals: Vec<String>,
}

impl SystemInfo {
    pub fn detect() -> Self {
        Self {
            os_release: OsRelease::load(),
            session: Session::detect(),
            hardware: Hardware::detect(),
            visible_globals: wayland::visible_globals(),
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

    pub fn current_distro<'a>(&self, catalog: &'a Catalog) -> Option<&'a Distro> {
        catalog
            .distros
            .iter()
            .find(|distro| self.is_current_distro(distro))
    }

    pub fn current_desktop<'a>(&self, catalog: &'a Catalog) -> Option<&'a Desktop> {
        catalog
            .desktops
            .iter()
            .find(|desktop| self.is_running(&desktop.session_names))
    }

    /// Checks `XDG_CURRENT_DESKTOP` first, then globals only one compositor offers.
    pub fn current_compositor<'a>(&self, catalog: &'a Catalog) -> Option<&'a Compositor> {
        let by_session = catalog
            .compositors
            .iter()
            .find(|compositor| self.is_running(&compositor.session_names));

        by_session.or_else(|| {
            catalog
                .compositors
                .iter()
                .find(|compositor| compositor.matches_fingerprint(&self.visible_globals))
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
        if let Some(desktop) = self.current_desktop(catalog) {
            return desktop.name.clone();
        }
        if let Some(compositor) = self.current_compositor(catalog) {
            return compositor.name.clone();
        }
        if let Some(name) = self.session.desktop_names.first() {
            return name.clone();
        }
        gettext("an unknown desktop")
    }
}
