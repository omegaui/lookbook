//! "Can I run it?" for one catalog item: what the user's computer means for it,
//! which ideas they would gain or lose, and how to try it. No UI code.
//!
//! Whether a desktop or compositor is packaged for the user's distro needs a
//! live Repology lookup; this module only says what to look up (`PackageLookup`).

use gettextrs::gettext;

use crate::catalog::{Catalog, Compositor, Desktop, Distro, ItemKind, ItemRef};
use crate::ideas::{evaluate, CurrentSetup};
use crate::repology::{resolve_repos, ResolvedRepo};
use crate::system::SystemInfo;

const GIB: u64 = 1024 * 1024 * 1024;
/// Below this, heavier desktops feel slow.
const COMFORTABLE_MEMORY_GIB: u64 = 8;
/// Below this, even light desktops struggle with a modern browser.
const MINIMUM_MEMORY_GIB: u64 = 4;
const BOXES_URL: &str = "https://flathub.org/apps/org.gnome.Boxes";
const IMPRESSION_URL: &str = "https://flathub.org/apps/io.gitlab.adhami3310.Impression";

/// How a check came out. Good shows a yellow tick, Note an info sign, Problem a warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckState {
    Good,
    Note,
    Problem,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckLine {
    pub state: CheckState,
    pub title: String,
    pub detail: String,
}

/// A way to try the item, with an optional link button.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TryStep {
    pub title: String,
    pub detail: String,
    pub link: Option<Link>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub label: String,
    pub url: String,
}

/// What to ask Repology, and where to look in its answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageLookup {
    /// For example "hyprland".
    pub project: String,
    /// The user's distro, for example "CachyOS".
    pub distro_name: String,
    pub repos: Vec<ResolvedRepo>,
    /// Desktops usually install as a group or meta-package, not one package.
    pub is_desktop: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunCheck {
    pub name: String,
    pub verdict: String,
    pub verdict_detail: String,
    pub is_running: bool,
    pub checks: Vec<CheckLine>,
    /// Ideas that are ready with the item but not with the user's setup.
    pub gained: Vec<String>,
    /// Ideas that are ready with the user's setup but not with the item.
    pub lost: Vec<String>,
    pub package: Option<PackageLookup>,
    pub try_steps: Vec<TryStep>,
}

pub fn check(item: &ItemRef, catalog: &Catalog, system: &SystemInfo) -> Option<RunCheck> {
    match item.kind {
        ItemKind::Distro => catalog
            .find_distro(&item.id)
            .map(|distro| check_distro(distro, catalog, system)),
        ItemKind::Desktop => catalog
            .find_desktop(&item.id)
            .map(|desktop| check_desktop(desktop, catalog, system)),
        ItemKind::Compositor => catalog
            .find_compositor(&item.id)
            .map(|compositor| check_compositor(compositor, catalog, system)),
    }
}

fn check_distro(distro: &Distro, catalog: &Catalog, system: &SystemInfo) -> RunCheck {
    let is_running = system.is_current_distro(distro);
    let (gained, lost) = match default_desktop_setup(distro, catalog) {
        Some(theirs) => idea_changes(&CurrentSetup::detect(catalog, system), &theirs, catalog),
        None => (Vec::new(), Vec::new()),
    };

    RunCheck {
        name: distro.name.clone(),
        verdict: distro_verdict(is_running, system),
        verdict_detail: gettext("It is a whole operating system, so the question is your hardware. Try it without installing first."),
        is_running,
        checks: hardware_checks(system, distro.nvidia_note.as_deref()),
        gained,
        lost,
        package: None,
        try_steps: distro_try_steps(distro),
    }
}

fn check_desktop(desktop: &Desktop, catalog: &Catalog, system: &SystemInfo) -> RunCheck {
    let is_running = system.runs_desktop(&desktop.id);
    let theirs = crate::details::setup_for(&ItemRef::new(ItemKind::Desktop, &desktop.id), catalog);
    let (gained, lost) = match theirs {
        Some(theirs) => idea_changes(&CurrentSetup::detect(catalog, system), &theirs, catalog),
        None => (Vec::new(), Vec::new()),
    };
    let nvidia_note = theirs_nvidia_note(desktop, catalog);
    let package = package_lookup(desktop.repology.as_deref(), true, catalog, system);

    RunCheck {
        name: desktop.name.clone(),
        verdict: session_verdict(is_running, package.is_some()),
        verdict_detail: session_verdict_detail(is_running),
        is_running,
        checks: hardware_checks(system, nvidia_note),
        gained,
        lost,
        package,
        try_steps: session_try_steps(&desktop.name, &desktop.website),
    }
}

fn check_compositor(compositor: &Compositor, catalog: &Catalog, system: &SystemInfo) -> RunCheck {
    let is_running = system.runs_compositor(&compositor.id);
    let theirs = CurrentSetup {
        desktop: None,
        compositor: Some(compositor),
    };
    let (gained, lost) = idea_changes(&CurrentSetup::detect(catalog, system), &theirs, catalog);
    let package = package_lookup(compositor.repology.as_deref(), false, catalog, system);

    RunCheck {
        name: compositor.name.clone(),
        verdict: session_verdict(is_running, package.is_some()),
        verdict_detail: session_verdict_detail(is_running),
        is_running,
        checks: hardware_checks(system, compositor.nvidia_note.as_deref()),
        gained,
        lost,
        package,
        try_steps: session_try_steps(&compositor.name, &compositor.website),
    }
}

fn distro_verdict(is_running: bool, system: &SystemInfo) -> String {
    if is_running {
        return gettext("You are running it right now.");
    }
    if memory_gib(system) < MINIMUM_MEMORY_GIB {
        return gettext("Yes, with a light desktop.");
    }
    gettext("Yes, your computer can run it.")
}

/// "Checking…" only when a package lookup will follow and settle the answer.
fn session_verdict(is_running: bool, will_look_up_package: bool) -> String {
    if is_running {
        gettext("You are running it right now.")
    } else if will_look_up_package {
        gettext("Checking your distro's packages…")
    } else {
        gettext("Most likely. Syspedia can't check your distro's packages, so see How to Try It.")
    }
}

fn session_verdict_detail(is_running: bool) -> String {
    if is_running {
        gettext("Everything below compares other choices against what you have.")
    } else {
        gettext("It installs next to what you use now; pick it on the login screen.")
    }
}

/// A desktop's NVIDIA note comes from the compositor it runs on.
fn theirs_nvidia_note<'a>(desktop: &Desktop, catalog: &'a Catalog) -> Option<&'a str> {
    catalog
        .compositor_of(desktop)
        .and_then(|compositor| compositor.nvidia_note.as_deref())
}

fn hardware_checks(system: &SystemInfo, nvidia_note: Option<&str>) -> Vec<CheckLine> {
    let mut checks = vec![memory_check(system)];
    checks.extend(graphics_check(system, nvidia_note));
    checks
}

/// The kernel reports a little less than what is installed (16 GB shows as
/// about 15.3), so round up to match the number on the box.
fn memory_gib(system: &SystemInfo) -> u64 {
    system.hardware.memory_bytes.div_ceil(GIB)
}

fn memory_check(system: &SystemInfo) -> CheckLine {
    let gib = memory_gib(system);
    let title = gettext("{} GB of memory").replace("{}", &gib.to_string());

    let (state, detail) = if gib >= COMFORTABLE_MEMORY_GIB {
        (CheckState::Good, gettext("Plenty for any desktop, browser tabs included."))
    } else if gib >= MINIMUM_MEMORY_GIB {
        (CheckState::Note, gettext("Enough for most desktops. GNOME and KDE Plasma feel smoother with 8 GB."))
    } else {
        (CheckState::Problem, gettext("Tight. A light desktop like Xfce or LXQt, or a tiling compositor, suits it best."))
    };

    CheckLine { state, title, detail }
}

/// Mesa drivers ship with every distro; NVIDIA needs its own driver.
fn graphics_check(system: &SystemInfo, nvidia_note: Option<&str>) -> Option<CheckLine> {
    let gpus = &system.hardware.gpus;
    if gpus.is_empty() {
        return None;
    }

    let nvidia = gpus.iter().find(|gpu| gpu.to_lowercase().contains("nvidia"));
    let line = match nvidia {
        Some(gpu) => CheckLine {
            state: CheckState::Note,
            title: gpu.clone(),
            detail: nvidia_note
                .map(str::to_owned)
                .unwrap_or_else(|| gettext("NVIDIA cards need NVIDIA's driver; most distros install it in a few steps.")),
        },
        None => CheckLine {
            state: CheckState::Good,
            title: gpus.join(", "),
            detail: gettext("Works out of the box with the open Mesa drivers."),
        },
    };
    Some(line)
}

/// The setup a distro starts with, for example Fedora Workstation → GNOME on Mutter.
fn default_desktop_setup<'a>(distro: &Distro, catalog: &'a Catalog) -> Option<CurrentSetup<'a>> {
    let desktop = catalog.default_desktop_of(distro)?;
    crate::details::setup_for(&ItemRef::new(ItemKind::Desktop, &desktop.id), catalog)
}

/// Idea names ready with `theirs` but not `yours`, and the other way round.
fn idea_changes(yours: &CurrentSetup<'_>, theirs: &CurrentSetup<'_>, catalog: &Catalog) -> (Vec<String>, Vec<String>) {
    let mut gained = Vec::new();
    let mut lost = Vec::new();

    for idea in &catalog.ideas {
        let ready_yours = evaluate(idea, yours, catalog).is_available();
        let ready_theirs = evaluate(idea, theirs, catalog).is_available();
        if ready_theirs && !ready_yours {
            gained.push(idea.name.clone());
        }
        if ready_yours && !ready_theirs {
            lost.push(idea.name.clone());
        }
    }
    (gained, lost)
}

fn package_lookup(project: Option<&str>, is_desktop: bool, catalog: &Catalog, system: &SystemInfo) -> Option<PackageLookup> {
    let project = project?;
    let distro = distro_for_packages(catalog, system)?;
    let repos = resolve_repos(distro, &system.os_release);
    // No repository to ask (Debian testing has no version, for example): a
    // lookup would only ever say "not found", so say nothing instead.
    if repos.is_empty() {
        return None;
    }

    Some(PackageLookup {
        project: project.to_owned(),
        distro_name: system.distro_name(),
        repos,
        is_desktop,
    })
}

/// The catalog distro whose repositories the user has: an exact match first,
/// then one with the same os-release ID (another edition), then a parent from ID_LIKE.
fn distro_for_packages<'a>(catalog: &'a Catalog, system: &SystemInfo) -> Option<&'a Distro> {
    if let Some(distro) = system.current_distro(catalog) {
        return Some(distro);
    }
    let os_release = &system.os_release;
    let same_id = |id: &str| catalog.distros.iter().find(|distro| distro.os_release_id == id);

    same_id(&os_release.id).or_else(|| os_release.id_like.iter().find_map(|id| same_id(id)))
}

fn distro_try_steps(distro: &Distro) -> Vec<TryStep> {
    vec![
        TryStep {
            title: gettext("Try it in a window"),
            detail: gettext("GNOME Boxes runs it as a virtual machine, without touching your system."),
            link: Some(Link {
                label: gettext("Get Boxes"),
                url: BOXES_URL.to_owned(),
            }),
        },
        TryStep {
            title: gettext("Try it from a USB stick"),
            detail: gettext("Write the image with Impression, then start your computer from the stick. Most distros run live, without installing."),
            link: Some(Link {
                label: gettext("Get Impression"),
                url: IMPRESSION_URL.to_owned(),
            }),
        },
        TryStep {
            title: gettext("Download {}").replace("{}", &distro.name),
            detail: gettext("Get the image from the project's own website."),
            link: Some(Link {
                label: gettext("Website"),
                url: distro.website.clone(),
            }),
        },
    ]
}

fn session_try_steps(name: &str, website: &str) -> Vec<TryStep> {
    vec![
        TryStep {
            title: gettext("Log out and pick it"),
            detail: gettext("After installing, choose {} on the login screen. Your current desktop stays as it is.")
                .replace("{}", name),
            link: None,
        },
        TryStep {
            title: gettext("Read the install guide"),
            detail: gettext("The project's own guide covers anything your distro needs."),
            link: Some(Link {
                label: gettext("Website"),
                url: website.to_owned(),
            }),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::{Hardware, OsRelease, RunningSession, Session};

    fn system(memory_gib: u64, gpus: &[&str], desktop: &str) -> SystemInfo {
        let mut system = SystemInfo {
            os_release: OsRelease {
                id: "cachyos".to_owned(),
                id_like: vec!["arch".to_owned()],
                ..OsRelease::default()
            },
            session: Session {
                desktop_names: vec![desktop.to_owned()],
                ..Session::default()
            },
            hardware: Hardware {
                memory_bytes: memory_gib * GIB,
                gpus: gpus.iter().map(|gpu| (*gpu).to_owned()).collect(),
                ..Hardware::default()
            },
            visible_globals: Vec::new(),
            running: RunningSession::default(),
        };
        system.recognise(&catalog());
        system
    }

    fn catalog() -> Catalog {
        let json = include_bytes!("../data/resources/catalog/catalog.json");
        Catalog::from_json(json).expect("the bundled catalog should parse")
    }

    #[test]
    fn little_memory_is_a_problem() {
        let line = memory_check(&system(2, &[], "Hyprland"));
        assert_eq!(line.state, CheckState::Problem);
    }

    #[test]
    fn nvidia_shows_the_item_note() {
        let line = graphics_check(&system(16, &["NVIDIA GeForce RTX 5050"], "Hyprland"), Some("Use the open modules.")).unwrap();
        assert_eq!(line.state, CheckState::Note);
        assert_eq!(line.detail, "Use the open modules.");
    }

    #[test]
    fn a_distro_syspedia_does_not_know_never_waits_on_packages() {
        let catalog = catalog();
        let mut system = system(16, &[], "Hyprland");
        system.os_release = OsRelease {
            id: "solus".to_owned(),
            ..OsRelease::default()
        };
        system.recognise(&catalog);
        let result = check(&ItemRef::new(ItemKind::Compositor, "niri"), &catalog, &system).unwrap();

        assert!(result.package.is_none());
        assert!(!result.verdict.contains("Checking"));
    }

    #[test]
    fn a_compositor_you_run_says_so() {
        let catalog = catalog();
        let item = ItemRef::new(ItemKind::Compositor, "hyprland");
        let result = check(&item, &catalog, &system(16, &[], "Hyprland")).unwrap();
        assert!(result.is_running);
    }

    #[test]
    fn packages_come_from_the_distro_or_its_parent() {
        let catalog = catalog();
        let item = ItemRef::new(ItemKind::Compositor, "niri");
        let result = check(&item, &catalog, &system(16, &[], "Hyprland")).unwrap();

        let lookup = result.package.unwrap();
        assert_eq!(lookup.project, "niri");
        assert_eq!(lookup.repos.first().map(|repo| repo.repo.as_str()), Some("arch"));
    }

    /// GNOME has its own lock screen and Hyprland tiles by itself, so neither
    /// shows up as something gained or given up.
    #[test]
    fn built_in_features_are_not_gained_or_lost() {
        let catalog = catalog();
        let item = ItemRef::new(ItemKind::Desktop, "gnome");
        let result = check(&item, &catalog, &system(16, &[], "Hyprland")).unwrap();

        assert!(!result.lost.iter().any(|idea| idea == "Lock screen"));
        assert!(!result.gained.iter().any(|idea| idea == "Window tiling"));
    }
}
