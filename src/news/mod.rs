//! "What's new this month": recent releases of the kernel, distros, desktops
//! and compositors, read live from each project's own feed. No UI code.

mod dates;
mod feeds;

use crate::catalog::{NewsSource, NewsSourceKind};

pub use dates::Day;

/// How far back "this month" reaches.
pub const WINDOW_DAYS: i64 = 31;
const PRERELEASE_WORDS: [&str; 4] = ["rc", "beta", "alpha", "pre"];

/// One release as a feed lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedRelease {
    /// For example "v0.57.1", "45" or "Wayfire 0.11".
    pub version: String,
    pub day: Day,
    pub link: Option<String>,
    /// A test release worth showing anyway, like a mainline kernel release candidate.
    pub preview: bool,
}

/// A release ready to show: which project, which version, when, and where to read about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    pub source: NewsSource,
    pub version: String,
    pub day: Day,
    pub link: String,
    pub preview: bool,
}

/// Reads one project's feed.
pub async fn fetch_releases(session: &soup::Session, source: &NewsSource) -> anyhow::Result<Vec<FeedRelease>> {
    let body = crate::http::get_bytes(session, &source.url).await?;
    match source.kind {
        NewsSourceKind::KernelOrg => feeds::parse_kernel_org(&body),
        NewsSourceKind::Endoflife => feeds::parse_endoflife(&body),
        NewsSourceKind::Atom => feeds::parse_atom(&body),
        NewsSourceKind::Rss => feeds::parse_rss(&body),
    }
}

/// The newest release of each track (stable, and preview where the feed has one)
/// from the last `WINDOW_DAYS`, skipping betas and release candidates.
pub fn recent_releases(source: &NewsSource, releases: Vec<FeedRelease>, today: Day) -> Vec<Release> {
    let recent: Vec<FeedRelease> = releases
        .into_iter()
        .filter(|release| is_recent(release.day, today))
        .filter(|release| release.preview || !is_prerelease(&release.version))
        .collect();

    let newest_of_track = |preview: bool| {
        recent
            .iter()
            .filter(|release| release.preview == preview)
            .max_by(|first, second| version_numbers(&first.version).cmp(&version_numbers(&second.version)))
            .map(|release| to_release(source, release))
    };
    [newest_of_track(false), newest_of_track(true)]
        .into_iter()
        .flatten()
        .collect()
}

fn to_release(source: &NewsSource, release: &FeedRelease) -> Release {
    Release {
        source: source.clone(),
        version: without_project_name(&release.version, &source.name),
        day: release.day,
        link: release.link.clone().unwrap_or_else(|| source.link.clone()),
        preview: release.preview,
    }
}

fn is_recent(day: Day, today: Day) -> bool {
    let age = day.days_until(today);
    (0..=WINDOW_DAYS).contains(&age)
}

/// "1.12-rc3", "51.rc" and "v2.0.0-beta.1" are test releases; "Epoch 1.11.0" is not.
fn is_prerelease(version: &str) -> bool {
    version
        .to_lowercase()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|part| PRERELEASE_WORDS.iter().any(|word| part.starts_with(word)))
}

/// "Wayfire 0.11" from the Wayfire feed becomes "0.11".
fn without_project_name(version: &str, name: &str) -> String {
    let prefix_length = name.len();
    let starts_with_name = version
        .get(..prefix_length)
        .is_some_and(|start| start.eq_ignore_ascii_case(name));
    if starts_with_name {
        return version[prefix_length..].trim().to_owned();
    }
    version.to_owned()
}

/// The numbers in a version, in order, so "6.8.2" sorts above "6.7.6" and "51.1" above "50.6".
fn version_numbers(version: &str) -> Vec<u32> {
    version
        .split(|character: char| !character.is_ascii_digit())
        .filter_map(|part| part.parse().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::NewsCategory;

    fn source(name: &str) -> NewsSource {
        NewsSource {
            id: name.to_lowercase(),
            name: name.to_owned(),
            category: NewsCategory::Compositor,
            kind: NewsSourceKind::Atom,
            url: String::new(),
            link: "https://example.org".to_owned(),
            logo: None,
            short: String::new(),
            color: String::new(),
        }
    }

    fn release(version: &str, month: u32, day: u32) -> FeedRelease {
        FeedRelease {
            version: version.to_owned(),
            day: Day { year: 2026, month, day },
            link: None,
            preview: false,
        }
    }

    const TODAY: Day = Day { year: 2026, month: 11, day: 7 };

    #[test]
    fn picks_the_highest_version_not_the_latest_date() {
        let releases = vec![release("51.1", 10, 14), release("50.6", 10, 14)];
        let recent = recent_releases(&source("GNOME"), releases, TODAY);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].version, "51.1");
    }

    #[test]
    fn skips_old_releases_and_release_candidates() {
        let releases = vec![release("1.12", 5, 25), release("1.13-rc1", 11, 1)];
        assert!(recent_releases(&source("sway"), releases, TODAY).is_empty());
    }

    #[test]
    fn drops_the_project_name_from_titles() {
        let releases = vec![release("Wayfire 0.12", 11, 2)];
        let recent = recent_releases(&source("Wayfire"), releases, TODAY);
        assert_eq!(recent[0].version, "0.12");
        assert_eq!(recent[0].link, "https://example.org");
    }

    #[test]
    fn epoch_is_not_a_prerelease() {
        assert!(!is_prerelease("COSMIC Epoch 1.11.0"));
        assert!(is_prerelease("51.rc"));
        assert!(is_prerelease("v2.0.0-beta.1"));
    }

    #[test]
    fn previews_are_kept_beside_stable_releases() {
        let mut preview = release("7.4-rc1", 11, 1);
        preview.preview = true;
        let releases = vec![release("7.3.3", 11, 6), preview];

        let recent = recent_releases(&source("Linux"), releases, TODAY);
        assert_eq!(recent.len(), 2);
    }
}
