//! Reads popular GNOME Shell extensions live from extensions.gnome.org.

use serde::{Deserialize, Deserializer};

use crate::http::get_bytes;

const API_URL: &str = "https://extensions.gnome.org/api/v1/extensions/?ordering=-downloads&page_size=";
const SITE_URL: &str = "https://extensions.gnome.org";

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Extension {
    pub name: String,
    pub description: String,
    pub creator: Creator,
    #[serde(deserialize_with = "number_from_text")]
    pub downloads: u64,
    /// A full image address, when the extension has a screenshot.
    pub screenshot: Option<String>,
    /// A path on extensions.gnome.org, for example "/extension/307/dash-to-dock/".
    pub link: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Creator {
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
struct Page {
    results: Vec<Extension>,
}

/// The API sends numbers as text, for example "12391487". Accepts either.
fn number_from_text<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumberOrText {
        Number(u64),
        Text(String),
    }

    match NumberOrText::deserialize(deserializer)? {
        NumberOrText::Number(number) => Ok(number),
        NumberOrText::Text(text) => text.parse().map_err(serde::de::Error::custom),
    }
}

impl Extension {
    pub fn page_url(&self) -> String {
        format!("{SITE_URL}{}", self.link)
    }
}

/// For example 12391487 becomes "12.4M" and 947512 becomes "948K".
pub fn compact_count(count: u64) -> String {
    match count {
        0..1_000 => count.to_string(),
        1_000..1_000_000 => format!("{:.0}K", count as f64 / 1_000.0),
        _ => format!("{:.1}M", count as f64 / 1_000_000.0),
    }
}

/// The most downloaded extensions, up to `count`.
pub async fn fetch_popular(session: &soup::Session, count: usize) -> anyhow::Result<Vec<Extension>> {
    let body = get_bytes(session, &format!("{API_URL}{count}")).await?;
    parse_page(&body)
}

fn parse_page(body: &[u8]) -> anyhow::Result<Vec<Extension>> {
    let page: Page = serde_json::from_slice(body)?;
    Ok(page.results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_read_short() {
        assert_eq!(compact_count(12_391_487), "12.4M");
        assert_eq!(compact_count(947_512), "948K");
        assert_eq!(compact_count(512), "512");
    }

    #[test]
    fn reads_the_api_page() {
        let body = br#"{"count": 1, "next": null, "previous": null, "results": [{
            "id": "307", "uuid": "dash-to-dock@micxgx.gmail.com", "name": "Dash to Dock",
            "creator": {"id": 12017, "username": "michele_g", "display_name": "michele_g"},
            "description": "A dock for the Gnome Shell.", "downloads": "12391487",
            "screenshot": "https://extensions.gnome.org/extension-data/screenshots/screenshot_307.jpg",
            "icon": "https://extensions.gnome.org/extension-data/icons/icon_307_2.png",
            "link": "/extension/307/dash-to-dock/"
        }]}"#;

        let extensions = parse_page(body).unwrap();

        assert_eq!(extensions[0].name, "Dash to Dock");
        assert_eq!(extensions[0].downloads, 12_391_487);
        assert_eq!(extensions[0].page_url(), "https://extensions.gnome.org/extension/307/dash-to-dock/");
    }
}
