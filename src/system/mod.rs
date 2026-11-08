//! Detects what the user runs. This module has no UI code.

mod gpu;
mod hardware;
mod os_release;
mod session;
mod wayland;

use gettextrs::gettext;

pub use hardware::{kernel_label, Hardware};
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
    /// The desktop and compositor the catalog recognised, set by `recognise`.
    pub running: RunningSession,
}

/// Catalog ids of the desktop and compositor the user runs, when known.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunningSession {
    pub desktop: Option<String>,
    pub compositor: Option<String>,
}

impl SystemInfo {
    /// Detects the system and recognises its desktop and compositor.
    pub fn detect_with(catalog: &Catalog) -> Self {
        let mut system = Self::detect();
        system.recognise(catalog);
        system
    }

    /// Remembers which catalog desktop and compositor are running, so cards
    /// can mark them without the catalog at hand.
    pub fn recognise(&mut self, catalog: &Catalog) {
        self.running = RunningSession {
            desktop: self.current_desktop(catalog).map(|desktop| desktop.id.clone()),
            compositor: self.current_compositor(catalog).map(|compositor| compositor.id.clone()),
        };
    }

    pub fn runs_desktop(&self, id: &str) -> bool {
        self.running.desktop.as_deref() == Some(id)
    }

    pub fn runs_compositor(&self, id: &str) -> bool {
        self.running.compositor.as_deref() == Some(id)
    }

    fn detect() -> Self {
        Self {
            os_release: OsRelease::load(),
            session: Session::detect(),
            hardware: Hardware::detect(),
            visible_globals: wayland::visible_globals(),
            running: RunningSession::default(),
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

    /// The first entry of `XDG_CURRENT_DESKTOP` that one of `items` answers to.
    /// The order matters: Budgie sets "Budgie:GNOME", and Budgie must win.
    fn first_running<'a, T>(&self, items: &'a [T], session_names: impl Fn(&T) -> &[String]) -> Option<&'a T> {
        self.session.desktop_names.iter().find_map(|running| {
            items.iter().find(|item| {
                session_names(item)
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(running))
            })
        })
    }

    /// The exact edition when the catalog has it; otherwise the first distro
    /// with the same ID, so Fedora KDE still counts as Fedora.
    pub fn current_distro<'a>(&self, catalog: &'a Catalog) -> Option<&'a Distro> {
        let exact = catalog
            .distros
            .iter()
            .find(|distro| self.is_current_distro(distro));
        exact.or_else(|| {
            catalog
                .distros
                .iter()
                .find(|distro| distro.os_release_id == self.os_release.id)
        })
    }

    pub fn current_desktop<'a>(&self, catalog: &'a Catalog) -> Option<&'a Desktop> {
        self.first_running(&catalog.desktops, |desktop| &desktop.session_names)
    }

    /// A desktop's own compositor when a desktop runs; otherwise checks
    /// `XDG_CURRENT_DESKTOP`, then globals only one compositor offers.
    pub fn current_compositor<'a>(&self, catalog: &'a Catalog) -> Option<&'a Compositor> {
        if let Some(desktop) = self.current_desktop(catalog) {
            return catalog
                .compositors
                .iter()
                .find(|compositor| compositor.name == desktop.compositor);
        }
        let by_session = self.first_running(&catalog.compositors, |compositor| &compositor.session_names);
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
