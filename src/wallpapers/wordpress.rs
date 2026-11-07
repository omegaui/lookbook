//! Reads wallpapers from a WordPress page: every image there carries a `srcset`
//! listing the sizes WordPress made, for example "kswirl-400x225.jpg 400w".

use super::{title_from_file_name, Wallpaper};
use crate::catalog::WallpaperSource;

/// Smaller images on the page are icons or post thumbnails, not wallpapers.
const MIN_WALLPAPER_WIDTH: u32 = 1280;
/// Thumbnails close to this width keep the grid light and sharp.
const THUMBNAIL_WIDTH: u32 = 400;

/// One entry of a `srcset`: an image address and its width.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SizedImage {
    url: String,
    width: u32,
}

pub fn parse_page(html: &str, source: &WallpaperSource) -> Vec<Wallpaper> {
    let mut wallpapers: Vec<Wallpaper> = Vec::new();

    for srcset in srcset_values(html) {
        let sizes = parse_srcset(srcset);
        let Some(wallpaper) = wallpaper_from_sizes(&sizes, source) else {
            continue;
        };

        let already_listed = wallpapers.iter().any(|known| known.full_url == wallpaper.full_url);
        if !already_listed {
            wallpapers.push(wallpaper);
        }
    }

    wallpapers
}

/// The text inside every `srcset="…"` attribute on the page.
fn srcset_values(html: &str) -> Vec<&str> {
    const START: &str = "srcset=\"";
    let mut values = Vec::new();
    let mut rest = html;

    while let Some(start) = rest.find(START) {
        let after_start = &rest[start + START.len()..];
        let Some(end) = after_start.find('"') else {
            break;
        };
        values.push(&after_start[..end]);
        rest = &after_start[end..];
    }

    values
}

/// "a.jpg 2560w, a-400x225.jpg 400w" becomes two sized images.
fn parse_srcset(srcset: &str) -> Vec<SizedImage> {
    srcset
        .split(',')
        .filter_map(|entry| {
            let (url, width) = entry.trim().rsplit_once(' ')?;
            let width = width.strip_suffix('w')?.parse().ok()?;
            Some(SizedImage {
                url: url.to_owned(),
                width,
            })
        })
        .collect()
}

fn wallpaper_from_sizes(sizes: &[SizedImage], source: &WallpaperSource) -> Option<Wallpaper> {
    let largest = sizes.iter().max_by_key(|image| image.width)?;
    if largest.width < MIN_WALLPAPER_WIDTH {
        return None;
    }

    let thumbnail = sizes
        .iter()
        .min_by_key(|image| image.width.abs_diff(THUMBNAIL_WIDTH))?;
    let file_name = largest.url.rsplit('/').next().unwrap_or(&largest.url);
    let title_source = file_name.replace("-scaled", "");

    Some(Wallpaper {
        title: title_from_file_name(&title_source),
        source_id: source.id.clone(),
        thumbnail_url: thumbnail.url.clone(),
        full_url: largest.url.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::WallpaperSourceKind;

    fn source() -> WallpaperSource {
        WallpaperSource {
            id: "linuxh2o".to_owned(),
            name: "LinuxH2O".to_owned(),
            kind: WallpaperSourceKind::WordpressPage,
            index_url: "https://example/wallpapers/".to_owned(),
            link: "https://example/wallpapers/".to_owned(),
            credit: String::new(),
        }
    }

    #[test]
    fn picks_the_largest_image_and_a_small_thumbnail() {
        let html = r#"<img src="x" srcset="https://e/kswirl-scaled.jpg 2560w, https://e/kswirl-768x432.jpg 768w, https://e/kswirl-400x225.jpg 400w" sizes="100vw">"#;

        let wallpapers = parse_page(html, &source());

        assert_eq!(wallpapers.len(), 1);
        assert_eq!(wallpapers[0].title, "Kswirl");
        assert_eq!(wallpapers[0].full_url, "https://e/kswirl-scaled.jpg");
        assert_eq!(wallpapers[0].thumbnail_url, "https://e/kswirl-400x225.jpg");
    }

    #[test]
    fn skips_small_images_and_duplicates() {
        let html = r#"
            <img srcset="https://e/avatar.png 96w, https://e/avatar-48x48.png 48w">
            <img srcset="https://e/sea.jpg 1920w, https://e/sea-400x225.jpg 400w">
            <img srcset="https://e/sea.jpg 1920w, https://e/sea-400x225.jpg 400w">
        "#;

        let wallpapers = parse_page(html, &source());

        assert_eq!(wallpapers.len(), 1);
        assert_eq!(wallpapers[0].title, "Sea");
    }
}
