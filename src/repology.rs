//! Asks Repology whether a project is packaged for the user's distro.
//!
//! Repology asks API users to send at most one request a second and to name
//! themselves in the User-Agent; `crate::http` does the naming, and
//! `wait_for_turn` does the spacing.

use std::cell::Cell;
use std::time::{Duration, Instant};

use gtk::glib;
use serde::Deserialize;

use crate::catalog::{Distro, RepologyRepo};
use crate::system::OsRelease;

const API_URL: &str = "https://repology.org/api/v1/project/";
const MIN_GAP: Duration = Duration::from_secs(1);

thread_local! {
    static LAST_REQUEST: Cell<Option<Instant>> = const { Cell::new(None) };
}

/// One package as Repology lists it. Only the fields the app uses.
#[derive(Debug, Clone, Deserialize)]
pub struct RepologyPackage {
    pub repo: String,
    pub srcname: Option<String>,
    pub binname: Option<String>,
    pub visiblename: Option<String>,
    pub version: String,
}

impl RepologyPackage {
    /// The name to install, for example "hyprland".
    pub fn install_name(&self) -> &str {
        self.binname
            .as_deref()
            .or(self.srcname.as_deref())
            .or(self.visiblename.as_deref())
            .unwrap_or_default()
    }
}

/// A distro repository with its placeholders filled in for this system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRepo {
    /// For example "fedora_42".
    pub repo: String,
    pub label: Option<String>,
    /// For example "sudo dnf install {package}".
    pub install: String,
}

/// A package found in one of the user's repositories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageMatch {
    pub repo_label: Option<String>,
    pub name: String,
    pub version: String,
    /// For example "sudo pacman -S hyprland".
    pub install_command: String,
}

/// Fills `{version}` and `{codename}` from os-release. A repository that needs
/// a value the system does not have is left out.
pub fn resolve_repos(distro: &Distro, os_release: &OsRelease) -> Vec<ResolvedRepo> {
    distro
        .repology_repos
        .iter()
        .filter_map(|repo| resolve_repo(repo, os_release))
        .collect()
}

fn resolve_repo(repo: &RepologyRepo, os_release: &OsRelease) -> Option<ResolvedRepo> {
    let mut name = repo.repo.clone();
    if name.contains("{version}") {
        let version = os_release.version_id.as_deref()?.replace('.', "_");
        name = name.replace("{version}", &version);
    }

    let mut install = repo.install.clone();
    if install.contains("{codename}") {
        let codename = os_release.version_codename.as_deref()?;
        install = install.replace("{codename}", codename);
    }

    Some(ResolvedRepo {
        repo: name,
        label: repo.label.clone(),
        install,
    })
}

/// Looks through the user's repositories in order and returns the first package
/// of `project` found, preferring the package named like the project.
pub fn find_package(
    packages: &[RepologyPackage],
    repos: &[ResolvedRepo],
    project: &str,
) -> Option<PackageMatch> {
    repos.iter().find_map(|repo| {
        let in_repo: Vec<&RepologyPackage> = packages
            .iter()
            .filter(|package| repo_matches(&repo.repo, &package.repo))
            .collect();
        let package = best_package(&in_repo, project)?;

        Some(PackageMatch {
            repo_label: repo.label.clone(),
            name: package.install_name().to_owned(),
            version: package.version.clone(),
            install_command: repo.install.replace("{package}", package.install_name()),
        })
    })
}

/// "alpine_3_22_1" (from VERSION_ID 3.22.1) matches Repology's "alpine_3_22".
fn repo_matches(wanted: &str, listed: &str) -> bool {
    wanted == listed || wanted.starts_with(&format!("{listed}_"))
}

fn best_package<'a>(packages: &[&'a RepologyPackage], project: &str) -> Option<&'a RepologyPackage> {
    let named_like_project = |name: &Option<String>| name.as_deref() == Some(project);
    packages
        .iter()
        .find(|package| named_like_project(&package.binname))
        .or_else(|| packages.iter().find(|package| named_like_project(&package.srcname)))
        .or_else(|| packages.first())
        .copied()
}

/// Reads every package Repology knows for `project`.
pub async fn fetch_project(session: &soup::Session, project: &str) -> anyhow::Result<Vec<RepologyPackage>> {
    wait_for_turn().await;
    let body = crate::http::get_bytes_over_http1(session, &format!("{API_URL}{project}")).await?;
    Ok(serde_json::from_slice(&body)?)
}

/// Keeps requests at least a second apart. The slot is booked before
/// waiting, so calls that overlap queue up instead of going out together.
async fn wait_for_turn() {
    let now = Instant::now();
    let start = LAST_REQUEST.with(|cell| {
        let start = next_slot(cell.get(), now);
        cell.set(Some(start));
        start
    });
    let wait = start.saturating_duration_since(now);
    if !wait.is_zero() {
        glib::timeout_future(wait).await;
    }
}

/// The earliest time the next request may go out.
fn next_slot(last: Option<Instant>, now: Instant) -> Instant {
    match last {
        Some(last) => (last + MIN_GAP).max(now),
        None => now,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_calls_get_slots_a_second_apart() {
        let now = Instant::now();
        let first = next_slot(None, now);
        let second = next_slot(Some(first), now);
        let third = next_slot(Some(second), now);
        assert_eq!(second - first, MIN_GAP);
        assert_eq!(third - second, MIN_GAP);
    }

    fn package(repo: &str, binname: &str, version: &str) -> RepologyPackage {
        RepologyPackage {
            repo: repo.to_owned(),
            srcname: Some("hyprland".to_owned()),
            binname: Some(binname.to_owned()),
            visiblename: None,
            version: version.to_owned(),
        }
    }

    fn repo(name: &str, label: Option<&str>, install: &str) -> ResolvedRepo {
        ResolvedRepo {
            repo: name.to_owned(),
            label: label.map(str::to_owned),
            install: install.to_owned(),
        }
    }

    #[test]
    fn prefers_the_package_named_like_the_project() {
        let packages = [
            package("arch", "hyprpm", "0.57.1"),
            package("arch", "hyprland", "0.57.1"),
        ];
        let repos = [repo("arch", None, "sudo pacman -S {package}")];

        let found = find_package(&packages, &repos, "hyprland").unwrap();
        assert_eq!(found.install_command, "sudo pacman -S hyprland");
    }

    #[test]
    fn falls_back_to_later_repositories() {
        let packages = [package("aur", "hyprland", "0.57.1")];
        let repos = [
            repo("arch", None, "sudo pacman -S {package}"),
            repo("aur", Some("AUR"), "paru -S {package}"),
        ];

        let found = find_package(&packages, &repos, "hyprland").unwrap();
        assert_eq!(found.repo_label.as_deref(), Some("AUR"));
        assert_eq!(found.install_command, "paru -S hyprland");
    }

    #[test]
    fn point_releases_match_their_series() {
        assert!(repo_matches("alpine_3_22_1", "alpine_3_22"));
        assert!(repo_matches("fedora_42", "fedora_42"));
        assert!(!repo_matches("debian_13", "debian_13_backports"));
    }

    #[test]
    fn fills_version_and_codename() {
        let os_release = OsRelease {
            version_id: Some("24.04".to_owned()),
            version_codename: Some("noble".to_owned()),
            ..OsRelease::default()
        };
        let template = RepologyRepo {
            repo: "ubuntu_{version}".to_owned(),
            label: None,
            install: "sudo apt install -t {codename}-backports {package}".to_owned(),
        };

        let resolved = resolve_repo(&template, &os_release).unwrap();
        assert_eq!(resolved.repo, "ubuntu_24_04");
        assert_eq!(resolved.install, "sudo apt install -t noble-backports {package}");
    }

    #[test]
    fn leaves_out_repositories_that_need_a_missing_version() {
        let template = RepologyRepo {
            repo: "fedora_{version}".to_owned(),
            label: None,
            install: "sudo dnf install {package}".to_owned(),
        };
        assert!(resolve_repo(&template, &OsRelease::default()).is_none());
    }
}
