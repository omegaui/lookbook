//! Reads wallpaper collections live from their sources.
//!
//! Linux & You never stores or re-hosts these images: it reads each source's
//! list when the Wallpapers page opens and loads images from the source itself.

mod copyparty;
mod http;
mod portal;
mod wordpress;

use gtk::{gdk, glib};

use crate::catalog::{WallpaperSource, WallpaperSourceKind};

pub use http::new_session;
pub use portal::{file_name_from_url, set_wallpaper};

/// One wallpaper, with addresses on its source's server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wallpaper {
    pub title: String,
    pub source_id: String,
    pub thumbnail_url: String,
    pub full_url: String,
}

/// Reads one source's current list of wallpapers.
pub async fn fetch_source(
    session: &soup::Session,
    source: &WallpaperSource,
) -> anyhow::Result<Vec<Wallpaper>> {
    match source.kind {
        WallpaperSourceKind::Copyparty => {
            let listing_url = format!("{}?ls", source.index_url);
            let body = http::get_bytes(session, &listing_url).await?;
            copyparty::parse_listing(&body, source)
        }
        WallpaperSourceKind::WordpressPage => {
            let body = http::get_bytes(session, &source.index_url).await?;
            Ok(wordpress::parse_page(&String::from_utf8_lossy(&body), source))
        }
    }
}

/// Downloads an image and decodes it for display.
pub async fn fetch_texture(session: &soup::Session, url: &str) -> anyhow::Result<gdk::Texture> {
    let bytes = fetch_bytes(session, url).await?;
    Ok(gdk::Texture::from_bytes(&bytes)?)
}

pub async fn fetch_bytes(session: &soup::Session, url: &str) -> anyhow::Result<glib::Bytes> {
    http::get_bytes(session, url).await
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
    fn titles_read_like_words() {
        assert_eq!(title_from_file_name("dark-forest-2.png"), "Dark forest 2");
        assert_eq!(title_from_file_name("3d_model.jpg"), "3d model");
    }
}
