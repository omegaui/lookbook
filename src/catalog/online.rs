//! The catalog published online by the Catalog workflow, so data stays current
//! between app releases.
//!
//! At startup the app reads the copy it downloaded last time, if that copy is
//! readable, has the schema this version understands, and is newer than the
//! catalog bundled in the app; otherwise it uses the bundled one. In the
//! background it checks for a new copy at most once a day, asking with
//! `If-None-Match` so an unchanged catalog costs almost nothing. A new copy is
//! used from the next launch.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gtk::glib;
use serde::{Deserialize, Serialize};
use soup::prelude::*;

use super::Catalog;

/// The catalog site; each schema version is published in its own folder.
const CATALOG_SITE: &str = "https://omegaui.github.io/lookbook";
/// The catalog schema this version of the app reads. The workflow publishes
/// other schema versions under other paths, and the app ignores them.
pub const SCHEMA_VERSION: u32 = 1;
const REFRESH_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const CACHE_FOLDER: &str = "lookbook";
const CATALOG_FILE: &str = "catalog.json";
const STATE_FILE: &str = "catalog-state.json";

/// What happened when the app checked for a new catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshOutcome {
    /// Checked less than a day ago.
    NotDue,
    /// The published catalog has not changed.
    Unchanged,
    /// A new catalog was saved; it is used from the next launch.
    Saved,
}

/// What the app remembers about its last check.
#[derive(Debug, Default, Serialize, Deserialize)]
struct RefreshState {
    etag: Option<String>,
    /// Seconds since 1970.
    checked_at: u64,
}

impl Catalog {
    /// The newest catalog available without the network.
    pub fn load() -> anyhow::Result<Self> {
        let bundled = Self::load_bundled()?;
        let Some(downloaded) = read_downloaded() else {
            return Ok(bundled);
        };
        if is_better_than(&downloaded, &bundled) {
            tracing::debug!("Using the catalog downloaded on {}", downloaded.generated_at);
            return Ok(downloaded);
        }
        Ok(bundled)
    }
}

/// A downloaded catalog is used only when this app can read it and it is
/// newer than the one the app ships with. Dates are ISO 8601 in UTC, so they
/// compare as text.
fn is_better_than(downloaded: &Catalog, bundled: &Catalog) -> bool {
    downloaded.schema_version == SCHEMA_VERSION && downloaded.generated_at > bundled.generated_at
}

fn read_downloaded() -> Option<Catalog> {
    let bytes = fs::read(cache_folder().join(CATALOG_FILE)).ok()?;
    match Catalog::from_json(&bytes) {
        Ok(catalog) => Some(catalog),
        Err(error) => {
            tracing::warn!("Ignoring the downloaded catalog: {error:#}");
            None
        }
    }
}

/// Downloads a new catalog when the last check was more than a day ago.
pub async fn refresh_downloaded(session: &soup::Session) -> anyhow::Result<RefreshOutcome> {
    let state = read_state();
    if !is_due(&state, now_seconds()) {
        return Ok(RefreshOutcome::NotDue);
    }
    download(session, state).await
}

/// Checks for a new catalog now, for Refresh Catalog in the main menu.
pub async fn download_now(session: &soup::Session) -> anyhow::Result<RefreshOutcome> {
    download(session, read_state()).await
}

/// Asks for the published catalog, sending the last ETag so an unchanged
/// catalog costs almost nothing, and saves a new one.
async fn download(session: &soup::Session, state: RefreshState) -> anyhow::Result<RefreshOutcome> {
    let url = catalog_url();
    let message = soup::Message::new("GET", &url)?;
    if let (Some(etag), Some(headers)) = (&state.etag, message.request_headers()) {
        headers.append("If-None-Match", etag);
    }
    let body = session
        .send_and_read_future(&message, glib::Priority::LOW)
        .await?;

    match message.status() {
        soup::Status::NotModified => {
            save_state(&RefreshState { checked_at: now_seconds(), ..state })?;
            Ok(RefreshOutcome::Unchanged)
        }
        soup::Status::Ok => {
            if let Err(error) = save_catalog(&body) {
                // Wait a day before trying again, instead of downloading the
                // same unreadable file on every launch.
                save_state(&RefreshState { checked_at: now_seconds(), ..state })?;
                return Err(error);
            }
            let etag = message
                .response_headers()
                .and_then(|headers| headers.one("ETag"))
                .map(|etag| etag.to_string());
            save_state(&RefreshState { etag, checked_at: now_seconds() })?;
            Ok(RefreshOutcome::Saved)
        }
        status => anyhow::bail!("{url} answered {status:?}"),
    }
}

fn is_due(state: &RefreshState, now: u64) -> bool {
    now.saturating_sub(state.checked_at) >= REFRESH_INTERVAL.as_secs()
}

/// Saves the catalog only if this app can read it, writing a temporary file
/// first so a half-written catalog is never left behind.
/// For example "https://omegaui.github.io/lookbook/v1/catalog.json". It
/// follows the schema version, so the two never drift apart.
fn catalog_url() -> String {
    format!("{CATALOG_SITE}/v{SCHEMA_VERSION}/catalog.json")
}

fn save_catalog(body: &[u8]) -> anyhow::Result<()> {
    save_catalog_in(&cache_folder(), body)
}

fn save_catalog_in(folder: &Path, body: &[u8]) -> anyhow::Result<()> {
    let catalog = Catalog::from_json(body)?;
    if catalog.schema_version != SCHEMA_VERSION {
        anyhow::bail!("the published catalog has schema {}", catalog.schema_version);
    }

    fs::create_dir_all(folder)?;
    let temporary = folder.join(format!("{CATALOG_FILE}.part"));
    fs::write(&temporary, body)?;
    fs::rename(&temporary, folder.join(CATALOG_FILE))?;
    Ok(())
}

fn read_state() -> RefreshState {
    fs::read(cache_folder().join(STATE_FILE))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn save_state(state: &RefreshState) -> anyhow::Result<()> {
    let folder = cache_folder();
    fs::create_dir_all(&folder)?;
    fs::write(folder.join(STATE_FILE), serde_json::to_vec(state)?)?;
    Ok(())
}

/// Inside Flatpak this is the app's own cache folder, under ~/.var/app.
fn cache_folder() -> PathBuf {
    glib::user_cache_dir().join(CACHE_FOLDER)
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundled() -> Catalog {
        let json = include_bytes!("../../data/resources/catalog/catalog.json");
        Catalog::from_json(json).expect("the bundled catalog should parse")
    }

    /// A temporary folder, removed when the test ends.
    struct TestFolder(PathBuf);

    impl TestFolder {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("lookbook-test-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            Self(path)
        }
    }

    impl Drop for TestFolder {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn a_bad_download_never_replaces_the_good_copy() {
        let folder = TestFolder::new("bad-download");
        let good = include_bytes!("../../data/resources/catalog/catalog.json");
        save_catalog_in(&folder.0, good).expect("the bundled catalog saves");

        assert!(save_catalog_in(&folder.0, b"<html>Not found</html>").is_err());
        let other_schema = String::from_utf8_lossy(good).replacen("\"schema_version\": 1", "\"schema_version\": 99", 1);
        assert!(save_catalog_in(&folder.0, other_schema.as_bytes()).is_err());

        let kept = fs::read(folder.0.join(CATALOG_FILE)).unwrap();
        assert_eq!(kept, good);
    }

    #[test]
    fn the_url_follows_the_schema_version() {
        assert!(catalog_url().ends_with(&format!("/v{SCHEMA_VERSION}/catalog.json")));
    }

    #[test]
    fn a_newer_download_wins() {
        let bundled = bundled();
        let mut downloaded = bundled.clone();
        downloaded.generated_at = "2999-01-01T00:00:00Z".to_owned();
        assert!(is_better_than(&downloaded, &bundled));
    }

    #[test]
    fn an_older_download_loses() {
        let bundled = bundled();
        let mut downloaded = bundled.clone();
        downloaded.generated_at = "2000-01-01T00:00:00Z".to_owned();
        assert!(!is_better_than(&downloaded, &bundled));
    }

    #[test]
    fn another_schema_is_ignored() {
        let bundled = bundled();
        let mut downloaded = bundled.clone();
        downloaded.generated_at = "2999-01-01T00:00:00Z".to_owned();
        downloaded.schema_version = SCHEMA_VERSION + 1;
        assert!(!is_better_than(&downloaded, &bundled));
    }

    #[test]
    fn checks_at_most_once_a_day() {
        let state = RefreshState { etag: None, checked_at: 1_000_000 };
        assert!(!is_due(&state, 1_000_000 + 60));
        assert!(is_due(&state, 1_000_000 + REFRESH_INTERVAL.as_secs()));
        assert!(is_due(&RefreshState::default(), 1_000_000));
    }
}
