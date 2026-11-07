//! Reads a copyparty file server's JSON listing (`?ls`).

use serde::Deserialize;

use super::{title_from_file_name, Wallpaper};
use crate::catalog::WallpaperSource;

const IMAGE_EXTENSIONS: [&str; 3] = ["png", "jpg", "jpeg"];

#[derive(Debug, Deserialize)]
struct Listing {
    files: Vec<ListedFile>,
}

#[derive(Debug, Deserialize)]
struct ListedFile {
    /// The file name, often with an access key, for example "dark-forest.png?k=qBA7".
    href: String,
    ext: String,
}

impl ListedFile {
    fn is_image(&self) -> bool {
        IMAGE_EXTENSIONS.contains(&self.ext.to_lowercase().as_str())
    }

    fn file_name(&self) -> &str {
        self.href.split('?').next().unwrap_or(&self.href)
    }
}

/// `body` is the JSON from `<index_url>?ls`.
pub fn parse_listing(body: &[u8], source: &WallpaperSource) -> anyhow::Result<Vec<Wallpaper>> {
    let listing: Listing = serde_json::from_slice(body)?;

    let wallpapers = listing
        .files
        .iter()
        .filter(|file| file.is_image())
        .map(|file| wallpaper_from_file(file, source))
        .collect();

    Ok(wallpapers)
}

fn wallpaper_from_file(file: &ListedFile, source: &WallpaperSource) -> Wallpaper {
    let full_url = format!("{}{}", source.index_url, file.href);
    let separator = if file.href.contains('?') { '&' } else { '?' };

    Wallpaper {
        title: title_from_file_name(file.file_name()),
        source_id: source.id.clone(),
        // copyparty makes a small JPEG thumbnail when asked with `th=j`.
        thumbnail_url: format!("{full_url}{separator}th=j"),
        full_url,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::WallpaperSourceKind;

    fn source() -> WallpaperSource {
        WallpaperSource {
            id: "orangc".to_owned(),
            name: "orangc's walls".to_owned(),
            kind: WallpaperSourceKind::Copyparty,
            index_url: "https://files.example/walls/".to_owned(),
            link: "https://example/walls".to_owned(),
            credit: String::new(),
        }
    }

    #[test]
    fn keeps_images_and_builds_thumbnail_urls() {
        let body = br#"{"dirs": [], "files": [
            {"href": "dark-forest.png?k=qBA7", "ext": "png", "sz": 1},
            {"href": "README.md?k=X3ML", "ext": "md", "sz": 1}
        ]}"#;

        let wallpapers = parse_listing(body, &source()).unwrap();

        assert_eq!(wallpapers.len(), 1);
        assert_eq!(wallpapers[0].title, "Dark forest");
        assert_eq!(wallpapers[0].full_url, "https://files.example/walls/dark-forest.png?k=qBA7");
        assert_eq!(wallpapers[0].thumbnail_url, "https://files.example/walls/dark-forest.png?k=qBA7&th=j");
    }
}
