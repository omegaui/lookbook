//! Decides which ideas are ready on the user's system, and where the others work.

use crate::catalog::{Catalog, Compositor, Desktop, Idea, IdeaRoute, RouteKind};
use crate::system::SystemInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdeaStatus {
    /// The desktop or compositor already has it, for example GNOME's lock screen.
    /// `by` names which one. `route` is how to build your own anyway, if possible.
    BuiltIn { by: String, route: Option<String> },
    /// Buildable here. `route` says how, for example "Layer shell", and
    /// `interface` names the protocol it uses, if any, for example "zwlr_layer_shell_v1".
    Ready { route: String, interface: Option<String> },
    /// Not buildable here. `needs` names what is missing, `places` where it works.
    WorksElsewhere { needs: String, places: Vec<String> },
}

impl IdeaStatus {
    /// The user has it: built in, or ready to build.
    pub fn is_available(&self) -> bool {
        matches!(self, Self::BuiltIn { .. } | Self::Ready { .. })
    }
}

/// What the user runs, looked up in the catalog.
pub struct CurrentSetup<'a> {
    pub desktop: Option<&'a Desktop>,
    pub compositor: Option<&'a Compositor>,
}

impl<'a> CurrentSetup<'a> {
    pub fn detect(catalog: &'a Catalog, system: &SystemInfo) -> Self {
        Self {
            desktop: system.current_desktop(catalog),
            compositor: system.current_compositor(catalog),
        }
    }
}

pub fn evaluate(idea: &Idea, setup: &CurrentSetup<'_>, catalog: &Catalog) -> IdeaStatus {
    let working_route = idea.routes.iter().find(|route| route_works(route, setup));

    if let Some(by) = built_in_by(idea, setup) {
        let route = working_route.map(|route| route.label.clone());
        return IdeaStatus::BuiltIn { by, route };
    }

    match working_route {
        Some(route) => IdeaStatus::Ready {
            route: route.label.clone(),
            interface: route.requires.first().cloned(),
        },
        None => IdeaStatus::WorksElsewhere {
            needs: first_route_label(idea),
            places: places_where_it_works(idea, catalog),
        },
    }
}

/// The name of the desktop, or else the compositor, that already includes the idea.
fn built_in_by(idea: &Idea, setup: &CurrentSetup<'_>) -> Option<String> {
    let includes = |id: &str| idea.built_into.iter().any(|built_into| built_into == id);

    if let Some(desktop) = setup.desktop.filter(|desktop| includes(&desktop.id)) {
        return Some(desktop.name.clone());
    }
    setup
        .compositor
        .filter(|compositor| includes(&compositor.id))
        .map(|compositor| compositor.name.clone())
}

fn route_works(route: &IdeaRoute, setup: &CurrentSetup<'_>) -> bool {
    match route.kind {
        RouteKind::Protocol => setup
            .compositor
            .is_some_and(|compositor| supports_all(compositor, &route.requires)),
        RouteKind::Desktop => setup
            .desktop
            .is_some_and(|desktop| route.desktops.contains(&desktop.id)),
        RouteKind::Portal => portal_works(route, setup),
    }
}

/// A portal route works when the compositor's portal backend provides that
/// portal. With no compositor known, or an older catalog that names no
/// portal, the app cannot tell, so it counts as working.
fn portal_works(route: &IdeaRoute, setup: &CurrentSetup<'_>) -> bool {
    match (&route.portal, setup.compositor) {
        (Some(portal), Some(compositor)) => compositor.has_portal(portal),
        _ => true,
    }
}

fn supports_all(compositor: &Compositor, globals: &[String]) -> bool {
    globals.iter().all(|global| compositor.supports(global))
}

fn first_route_label(idea: &Idea) -> String {
    idea.routes
        .first()
        .map(|route| route.label.clone())
        .unwrap_or_default()
}

fn compositors_with_portal<'a>(route: &IdeaRoute, catalog: &'a Catalog) -> Vec<&'a str> {
    let Some(portal) = &route.portal else {
        return Vec::new();
    };
    catalog
        .compositors
        .iter()
        .filter(|compositor| compositor.has_portal(portal))
        .map(|compositor| compositor.name.as_str())
        .collect()
}

/// Compositors that offer a protocol route, then desktops named in desktop routes.
fn places_where_it_works(idea: &Idea, catalog: &Catalog) -> Vec<String> {
    let mut places: Vec<String> = Vec::new();

    for route in &idea.routes {
        let names: Vec<&str> = match route.kind {
            RouteKind::Protocol => compositors_supporting(route, catalog),
            RouteKind::Desktop => desktops_named(route, catalog),
            RouteKind::Portal => compositors_with_portal(route, catalog),
        };

        for name in names {
            if !places.iter().any(|place| place == name) {
                places.push(name.to_owned());
            }
        }
    }

    places
}

fn compositors_supporting<'a>(route: &IdeaRoute, catalog: &'a Catalog) -> Vec<&'a str> {
    catalog
        .compositors
        .iter()
        .filter(|compositor| supports_all(compositor, &route.requires))
        .map(|compositor| compositor.name.as_str())
        .collect()
}

fn desktops_named<'a>(route: &IdeaRoute, catalog: &'a Catalog) -> Vec<&'a str> {
    route
        .desktops
        .iter()
        .filter_map(|id| catalog.find_desktop(id))
        .map(|desktop| desktop.name.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::Session;

    fn bundled_catalog() -> Catalog {
        let json = include_bytes!("../data/resources/catalog/catalog.json");
        Catalog::from_json(json).expect("the bundled catalog should parse")
    }

    fn system_running(desktop_names: &[&str]) -> SystemInfo {
        SystemInfo {
            session: Session {
                desktop_names: desktop_names.iter().map(|name| name.to_string()).collect(),
                session_type: "wayland".to_owned(),
            },
            ..SystemInfo::default()
        }
    }

    fn status_of(idea_id: &str, desktop_names: &[&str]) -> IdeaStatus {
        let catalog = bundled_catalog();
        let system = system_running(desktop_names);
        let setup = CurrentSetup::detect(&catalog, &system);
        let idea = catalog.ideas.iter().find(|idea| idea.id == idea_id).unwrap();
        evaluate(idea, &setup, &catalog)
    }

    #[test]
    fn gnome_has_a_status_bar_and_extensions_can_replace_it() {
        let status = status_of("status-bar", &["GNOME"]);
        assert_eq!(
            status,
            IdeaStatus::BuiltIn {
                by: "GNOME".to_owned(),
                route: Some("Top bar extension (GJS)".to_owned()),
            }
        );
    }

    #[test]
    fn hyprland_tiles_by_itself() {
        let status = status_of("tiling", &["Hyprland"]);
        assert!(matches!(status, IdeaStatus::BuiltIn { by, .. } if by == "Hyprland"));
    }

    #[test]
    fn status_bar_on_hyprland_uses_layer_shell() {
        let status = status_of("status-bar", &["Hyprland"]);
        assert_eq!(
            status,
            IdeaStatus::Ready {
                route: "Layer shell".to_owned(),
                interface: Some("zwlr_layer_shell_v1".to_owned()),
            }
        );
    }

    #[test]
    fn gnome_has_its_own_lock_screen() {
        let status = status_of("lock-screen", &["GNOME"]);
        assert_eq!(status, IdeaStatus::BuiltIn { by: "GNOME".to_owned(), route: None });
    }

    #[test]
    fn a_custom_lock_screen_needs_session_lock() {
        let IdeaStatus::WorksElsewhere { places, .. } = status_of("lock-screen", &[]) else {
            panic!("an unknown session offers no session lock");
        };
        assert!(places.iter().any(|place| place == "sway"));
        assert!(!places.iter().any(|place| place == "Mutter"));
    }

    #[test]
    fn screenshots_need_the_screenshot_portal() {
        assert!(status_of("screenshot", &["GNOME"]).is_available());

        // A compositor whose portal backend has ScreenCast but no Screenshot,
        // and no capture protocols: only the portal route is left to judge.
        let catalog = bundled_catalog();
        let mut compositor = catalog.find_compositor("sway").unwrap().clone();
        compositor.protocols.globals.clear();
        compositor.portals = vec!["ScreenCast".to_owned()];
        let setup = CurrentSetup {
            desktop: None,
            compositor: Some(&compositor),
        };
        let idea = |id: &str| catalog.ideas.iter().find(|idea| idea.id == id).unwrap();
        assert!(!evaluate(idea("screenshot"), &setup, &catalog).is_available());
        assert!(evaluate(idea("recorder"), &setup, &catalog).is_available());
    }

    #[test]
    fn an_unknown_compositor_cannot_rule_a_portal_out() {
        assert!(status_of("screenshot", &[]).is_available());
    }
}
