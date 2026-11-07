//! Decides which ideas are ready on the user's system, and where the others work.

use crate::catalog::{Catalog, Compositor, Desktop, Idea, IdeaRoute, RouteKind};
use crate::system::SystemInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdeaStatus {
    /// Buildable here. `route` says how, for example "Shell extension (GJS)".
    Ready { route: String },
    /// Not buildable here. `needs` names what is missing, `places` where it works.
    WorksElsewhere { needs: String, places: Vec<String> },
}

impl IdeaStatus {
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
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

    match working_route {
        Some(route) => IdeaStatus::Ready {
            route: route.label.clone(),
        },
        None => IdeaStatus::WorksElsewhere {
            needs: first_route_label(idea),
            places: places_where_it_works(idea, catalog),
        },
    }
}

fn route_works(route: &IdeaRoute, setup: &CurrentSetup<'_>) -> bool {
    match route.kind {
        RouteKind::Protocol => setup
            .compositor
            .is_some_and(|compositor| supports_all(compositor, &route.requires)),
        RouteKind::Desktop => setup
            .desktop
            .is_some_and(|desktop| route.desktops.contains(&desktop.id)),
        RouteKind::Portal => true,
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

/// Compositors that offer a protocol route, then desktops named in desktop routes.
fn places_where_it_works(idea: &Idea, catalog: &Catalog) -> Vec<String> {
    let mut places: Vec<String> = Vec::new();

    for route in &idea.routes {
        let names: Vec<&str> = match route.kind {
            RouteKind::Protocol => compositors_supporting(route, catalog),
            RouteKind::Desktop => desktops_named(route, catalog),
            RouteKind::Portal => Vec::new(),
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
    fn status_bar_on_gnome_is_a_shell_extension() {
        let status = status_of("status-bar", &["GNOME"]);
        assert_eq!(status, IdeaStatus::Ready { route: "Top bar extension (GJS)".to_owned() });
    }

    #[test]
    fn status_bar_on_hyprland_uses_layer_shell() {
        let status = status_of("status-bar", &["Hyprland"]);
        assert_eq!(status, IdeaStatus::Ready { route: "Layer shell".to_owned() });
    }

    #[test]
    fn lock_screen_on_gnome_works_elsewhere() {
        let IdeaStatus::WorksElsewhere { places, .. } = status_of("lock-screen", &["GNOME"]) else {
            panic!("a custom lock screen should not be ready on GNOME");
        };
        assert!(places.iter().any(|place| place == "sway"));
        assert!(!places.iter().any(|place| place == "Mutter"));
    }

    #[test]
    fn screenshots_always_work_through_the_portal() {
        assert!(status_of("screenshot", &["GNOME"]).is_ready());
        assert!(status_of("screenshot", &[]).is_ready());
    }
}
