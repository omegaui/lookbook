//! What a details page compares and suggests: the item against the user's
//! system, similar items, and the ideas the item supports. No UI code.

use gettextrs::gettext;

use crate::catalog::{Catalog, Compositor, Desktop, Distro, ItemKind, ItemRef};
use crate::ideas::{evaluate, CurrentSetup};
use crate::system::SystemInfo;

const SIMILAR_COUNT: usize = 3;

/// One line of "Versus your system", for example "Packages: pacman | dnf".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparisonRow {
    pub label: String,
    pub yours: String,
    pub theirs: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comparison {
    /// What the user runs, for example "CachyOS".
    pub yours_name: String,
    pub theirs_name: String,
    pub rows: Vec<ComparisonRow>,
    /// The item is what the user already runs.
    pub is_yours: bool,
}

fn row(label: String, yours: &str, theirs: &str) -> ComparisonRow {
    ComparisonRow {
        label,
        yours: yours.to_owned(),
        theirs: theirs.to_owned(),
    }
}

pub fn compare(item: &ItemRef, catalog: &Catalog, system: &SystemInfo) -> Option<Comparison> {
    match item.kind {
        ItemKind::Distro => catalog
            .find_distro(&item.id)
            .map(|distro| compare_distro(distro, catalog, system)),
        ItemKind::Desktop => catalog
            .find_desktop(&item.id)
            .map(|desktop| compare_desktop(desktop, catalog, system)),
        ItemKind::Compositor => catalog
            .find_compositor(&item.id)
            .map(|compositor| compare_compositor(compositor, catalog, system)),
    }
}

fn compare_distro(distro: &Distro, catalog: &Catalog, system: &SystemInfo) -> Comparison {
    let yours = system.current_distro(catalog);
    let unknown = gettext("Unknown");
    let yours_value = |pick: fn(&Distro) -> String| yours.map_or(unknown.clone(), pick);

    Comparison {
        yours_name: system.distro_name(),
        theirs_name: distro.name.clone(),
        rows: vec![
            row(gettext("Based on"), &yours_value(|d| d.family.label()), &distro.family.label()),
            row(gettext("Packages"), &yours_value(|d| d.package_manager.clone()), &distro.package_manager),
            row(gettext("Releases"), &yours_value(|d| d.release.label()), &distro.release.label()),
            row(gettext("Desktop"), &system.desktop_name(catalog), &distro.default_desktop),
        ],
        is_yours: system.is_current_distro(distro),
    }
}

fn compare_desktop(desktop: &Desktop, catalog: &Catalog, system: &SystemInfo) -> Comparison {
    let yours = CurrentSetup::detect(catalog, system);
    let theirs = setup_for_desktop(desktop, catalog);
    let unknown = gettext("Unknown");

    Comparison {
        yours_name: system.desktop_name(catalog),
        theirs_name: desktop.name.clone(),
        rows: vec![
            row(
                gettext("Compositor"),
                yours.compositor.map_or(unknown.as_str(), |compositor| compositor.name.as_str()),
                &desktop.compositor,
            ),
            row(gettext("Toolkit"), &your_toolkit(&yours), &desktop.toolkit),
            row(gettext("Ideas you have"), &ready_count(&yours, catalog), &ready_count(&theirs, catalog)),
        ],
        is_yours: system.runs_desktop(&desktop.id),
    }
}

/// A desktop names its toolkit. A compositor on its own runs apps from any toolkit.
fn your_toolkit(yours: &CurrentSetup<'_>) -> String {
    match (yours.desktop, yours.compositor) {
        (Some(desktop), _) => desktop.toolkit.clone(),
        (None, Some(_)) => gettext("Your choice"),
        (None, None) => gettext("Unknown"),
    }
}

fn compare_compositor(compositor: &Compositor, catalog: &Catalog, system: &SystemInfo) -> Comparison {
    let yours = CurrentSetup::detect(catalog, system);
    let theirs = CurrentSetup {
        desktop: None,
        compositor: Some(compositor),
    };
    let unknown = gettext("Unknown");

    Comparison {
        yours_name: yours
            .compositor
            .map_or(unknown.clone(), |compositor| compositor.name.clone()),
        theirs_name: compositor.name.clone(),
        rows: vec![
            row(
                gettext("Layout"),
                &yours.compositor.map_or(unknown.clone(), |compositor| compositor.kind.label()),
                &compositor.kind.label(),
            ),
            row(
                gettext("Built on"),
                yours.compositor.map_or(unknown.as_str(), |compositor| compositor.base.as_str()),
                &compositor.base,
            ),
            row(gettext("Ideas you have"), &ready_count(&yours, catalog), &ready_count(&theirs, catalog)),
        ],
        is_yours: system.runs_compositor(&compositor.id),
    }
}

/// For example "14 of 16".
fn ready_count(setup: &CurrentSetup<'_>, catalog: &Catalog) -> String {
    let ready = catalog
        .ideas
        .iter()
        .filter(|idea| evaluate(idea, setup, catalog).is_available())
        .count();

    gettext("{ready} of {total}")
        .replace("{ready}", &ready.to_string())
        .replace("{total}", &catalog.ideas.len().to_string())
}

/// The setup an item stands for, to judge ideas against it. Distros have none.
pub fn setup_for<'a>(item: &ItemRef, catalog: &'a Catalog) -> Option<CurrentSetup<'a>> {
    match item.kind {
        ItemKind::Distro => None,
        ItemKind::Desktop => catalog
            .find_desktop(&item.id)
            .map(|desktop| setup_for_desktop(desktop, catalog)),
        ItemKind::Compositor => catalog.find_compositor(&item.id).map(|compositor| CurrentSetup {
            desktop: None,
            compositor: Some(compositor),
        }),
    }
}

/// A desktop runs on its compositor, for example Budgie on labwc.
fn setup_for_desktop<'a>(desktop: &'a Desktop, catalog: &'a Catalog) -> CurrentSetup<'a> {
    CurrentSetup {
        desktop: Some(desktop),
        compositor: catalog.compositor_of(desktop),
    }
}

/// Up to three items like this one: the same distro family, the same compositor
/// layout, or the same desktop toolkit.
pub fn similar_items(item: &ItemRef, catalog: &Catalog) -> Vec<ItemRef> {
    let ids: Vec<&str> = match item.kind {
        ItemKind::Distro => similar_distros(&item.id, catalog),
        ItemKind::Desktop => similar_desktops(&item.id, catalog),
        ItemKind::Compositor => similar_compositors(&item.id, catalog),
    };

    ids.into_iter()
        .take(SIMILAR_COUNT)
        .map(|id| ItemRef::new(item.kind, id))
        .collect()
}

fn similar_distros<'a>(id: &str, catalog: &'a Catalog) -> Vec<&'a str> {
    let Some(distro) = catalog.find_distro(id) else {
        return Vec::new();
    };
    let others = catalog.distros.iter().filter(|other| other.id != id);
    let same_family = others.clone().filter(|other| other.family == distro.family);
    let same_release = others.filter(|other| other.family != distro.family && other.release == distro.release);

    same_family.chain(same_release).map(|other| other.id.as_str()).collect()
}

fn similar_desktops<'a>(id: &str, catalog: &'a Catalog) -> Vec<&'a str> {
    let Some(desktop) = catalog.find_desktop(id) else {
        return Vec::new();
    };
    catalog
        .desktops
        .iter()
        .filter(|other| other.id != id && other.toolkit_family == desktop.toolkit_family)
        .map(|other| other.id.as_str())
        .collect()
}

fn similar_compositors<'a>(id: &str, catalog: &'a Catalog) -> Vec<&'a str> {
    let Some(compositor) = catalog.find_compositor(id) else {
        return Vec::new();
    };
    catalog
        .compositors
        .iter()
        .filter(|other| other.id != id && other.kind == compositor.kind)
        .map(|other| other.id.as_str())
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
        let mut system = SystemInfo {
            session: Session {
                desktop_names: desktop_names.iter().map(|name| name.to_string()).collect(),
                session_type: "wayland".to_owned(),
            },
            ..SystemInfo::default()
        };
        system.recognise(&bundled_catalog());
        system
    }

    #[test]
    fn budgie_wins_over_the_gnome_it_names_after_itself() {
        let catalog = bundled_catalog();
        let system = system_running(&["Budgie", "GNOME"]);

        assert_eq!(system.current_desktop(&catalog).map(|desktop| desktop.id.as_str()), Some("budgie"));
        assert_eq!(system.current_compositor(&catalog).map(|compositor| compositor.id.as_str()), Some("labwc"));
        assert!(!system.runs_desktop("gnome"));
        assert!(!system.runs_compositor("mutter"));
    }

    #[test]
    fn a_fedora_edition_the_catalog_lacks_still_counts_as_fedora() {
        let catalog = bundled_catalog();
        let system = SystemInfo {
            os_release: crate::system::OsRelease {
                id: "fedora".to_owned(),
                variant_id: Some("kde".to_owned()),
                ..Default::default()
            },
            ..SystemInfo::default()
        };
        assert_eq!(system.current_distro(&catalog).map(|distro| distro.id.as_str()), Some("fedora"));
    }

    #[test]
    fn ubuntu_runs_gnome() {
        let system = system_running(&["ubuntu", "GNOME"]);
        assert!(system.runs_desktop("gnome"));
        assert!(system.runs_compositor("mutter"));
    }

    #[test]
    fn budgie_ideas_use_labwc() {
        let catalog = bundled_catalog();
        let setup = setup_for(&ItemRef::new(ItemKind::Desktop, "budgie"), &catalog).unwrap();

        assert_eq!(setup.compositor.map(|compositor| compositor.id.as_str()), Some("labwc"));
    }

    #[test]
    fn similar_distros_share_a_family_first() {
        let catalog = bundled_catalog();
        let similar = similar_items(&ItemRef::new(ItemKind::Distro, "cachyos"), &catalog);

        assert_eq!(similar[0].id, "endeavouros");
        assert!(similar.iter().all(|item| item.id != "cachyos"));
    }

    #[test]
    fn comparing_the_running_compositor_says_so() {
        let catalog = bundled_catalog();
        let system = system_running(&["Hyprland"]);
        let comparison = compare(&ItemRef::new(ItemKind::Compositor, "hyprland"), &catalog, &system).unwrap();

        assert!(comparison.is_yours);
        assert_eq!(comparison.rows.last().unwrap().yours, comparison.rows.last().unwrap().theirs);
    }
}
