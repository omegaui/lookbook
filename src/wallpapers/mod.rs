//! Reads wallpaper collections live from their sources.
//!
//! Lookbook never stores or re-hosts these images: it reads each source's
//! list when the Wallpapers page opens and loads images from the source itself.

mod copyparty;
mod portal;
mod wordpress;

use gettextrs::gettext;
use gtk::glib;

use crate::catalog::{Catalog, WallpaperSource, WallpaperSourceKind};
use crate::system::SystemInfo;

/// The portal interface apps use to set the background.
const WALLPAPER_PORTAL: &str = "Wallpaper";

pub use crate::images::fetch_texture;
pub use portal::{file_name_from_url, set_wallpaper, WallpaperOutcome};

/// One wallpaper, with addresses on its source's server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wallpaper {
    pub title: String,
    pub source_id: String,
    pub thumbnail_url: String,
    pub full_url: String,
}

/// A source's wallpapers, read once per launch: the Explore spotlight and
/// the Wallpapers page share the list instead of each downloading it.
pub async fn fetch_source(
    session: &soup::Session,
    source: &WallpaperSource,
) -> anyhow::Result<Vec<Wallpaper>> {
    if let Some(wallpapers) = SOURCE_LISTS.with(|lists| lists.borrow().get(&source.id).cloned()) {
        return Ok(wallpapers);
    }
    let wallpapers = read_source(session, source).await?;
    SOURCE_LISTS.with(|lists| lists.borrow_mut().insert(source.id.clone(), wallpapers.clone()));
    Ok(wallpapers)
}

thread_local! {
    static SOURCE_LISTS: std::cell::RefCell<std::collections::HashMap<String, Vec<Wallpaper>>> =
        std::cell::RefCell::default();
}

async fn read_source(session: &soup::Session, source: &WallpaperSource) -> anyhow::Result<Vec<Wallpaper>> {
    match source.kind {
        WallpaperSourceKind::Copyparty => {
            let listing_url = format!("{}?ls", source.index_url);
            let body = crate::http::get_bytes(session, &listing_url).await?;
            copyparty::parse_listing(&body, source)
        }
        WallpaperSourceKind::WordpressPage => {
            let body = crate::http::get_bytes(session, &source.index_url).await?;
            Ok(wordpress::parse_page(&String::from_utf8_lossy(&body), source))
        }
    }
}

/// Drops the wallpapers whose owners asked to be left out.
pub fn remove_blocked(wallpapers: Vec<Wallpaper>, blocklist: &[String]) -> Vec<Wallpaper> {
    let blocked: Vec<&str> = blocklist.iter().map(|url| without_query(url)).collect();
    wallpapers
        .into_iter()
        .filter(|wallpaper| !blocked.contains(&without_query(&wallpaper.full_url)))
        .collect()
}

/// Copyparty adds an access key ("?k=…") that can change; the takedown
/// list must match the image, not the key.
fn without_query(url: &str) -> &str {
    url.split('?').next().unwrap_or(url)
}

/// A note for users whose compositor has no wallpaper portal, so Set as Wallpaper
/// may do nothing there.
pub fn portal_hint(catalog: &Catalog, system: &SystemInfo) -> Option<String> {
    let compositor = system.current_compositor(catalog)?;
    if compositor.has_portal(WALLPAPER_PORTAL) {
        return None;
    }

    Some(
        gettext("{} usually cannot set wallpapers from apps. Save the image, then set it with your wallpaper tool, like hyprpaper, swaybg or awww.")
            .replace("{}", &compositor.name),
    )
}

/// Downloads an image and decodes it for display.

pub async fn fetch_bytes(session: &soup::Session, url: &str) -> anyhow::Result<glib::Bytes> {
    crate::http::get_bytes(session, url).await
}

/// Turns a file name like "dark-forest-2.png" into "Dark forest 2".
fn title_from_file_name(file_name: &str) -> String {
    let stem = file_name.rsplit_once('.').map_or(file_name, |(stem, _)| stem);
    let words = stem.replace(['-', '_'], " ");
    let mut characters = words.trim().chars();

    match characters.next() {
        Some(first) => first.to_uppercase().chain(characters).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn takedowns_match_whatever_the_access_key() {
        let wallpaper = Wallpaper {
            title: "Sea".to_owned(),
            source_id: "orangc".to_owned(),
            thumbnail_url: String::new(),
            full_url: "https://e/walls/sea.png?k=NEW".to_owned(),
        };
        let kept = remove_blocked(vec![wallpaper], &["https://e/walls/sea.png?k=OLD".to_owned()]);
        assert!(kept.is_empty());
    }

    #[test]
    fn titles_read_like_words() {
        assert_eq!(title_from_file_name("dark-forest-2.png"), "Dark forest 2");
        assert_eq!(title_from_file_name("3d_model.jpg"), "3d model");
    }
}
