//! Reads releases out of each kind of feed. Every function takes the feed's
//! text and returns every release it lists; picking the recent ones happens later.

use serde::Deserialize;

use super::dates::Day;
use super::FeedRelease;

const ATOM_NAMESPACE: &str = "http://www.w3.org/2005/Atom";

/// kernel.org's `releases.json`: the newest mainline and stable kernels.
/// Longterm point releases come out weekly and would crowd everything else.
pub fn parse_kernel_org(body: &[u8]) -> anyhow::Result<Vec<FeedRelease>> {
    #[derive(Deserialize)]
    struct Releases {
        releases: Vec<KernelRelease>,
    }
    #[derive(Deserialize)]
    struct KernelRelease {
        moniker: String,
        version: String,
        released: Released,
    }
    #[derive(Deserialize)]
    struct Released {
        isodate: String,
    }

    let releases: Releases = serde_json::from_slice(body)?;
    let wanted = releases.releases.into_iter().filter_map(|release| {
        let preview = match release.moniker.as_str() {
            "mainline" => true,
            "stable" => false,
            _ => return None,
        };
        Some(FeedRelease {
            version: release.version,
            day: Day::from_iso(&release.released.isodate)?,
            link: None,
            preview,
        })
    });
    Ok(wanted.collect())
}

/// The endoflife.date API: each release cycle's first release and its latest point release.
pub fn parse_endoflife(body: &[u8]) -> anyhow::Result<Vec<FeedRelease>> {
    #[derive(Deserialize)]
    struct Answer {
        result: Product,
    }
    #[derive(Deserialize)]
    struct Product {
        releases: Vec<Cycle>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Cycle {
        name: String,
        release_date: Option<String>,
        latest: Option<Latest>,
    }
    #[derive(Deserialize)]
    struct Latest {
        name: String,
        date: Option<String>,
        link: Option<String>,
    }

    let answer: Answer = serde_json::from_slice(body)?;
    let mut releases = Vec::new();
    for cycle in answer.result.releases {
        let link = cycle.latest.as_ref().and_then(|latest| latest.link.clone());
        if let Some(day) = cycle.release_date.as_deref().and_then(Day::from_iso) {
            releases.push(FeedRelease { version: cycle.name, day, link: link.clone(), preview: false });
        }
        if let Some(latest) = cycle.latest {
            if let Some(day) = latest.date.as_deref().and_then(Day::from_iso) {
                releases.push(FeedRelease { version: latest.name, day, link, preview: false });
            }
        }
    }
    Ok(releases)
}

/// An Atom feed: each entry's title, date and link.
pub fn parse_atom(body: &[u8]) -> anyhow::Result<Vec<FeedRelease>> {
    let text = std::str::from_utf8(body)?;
    let document = roxmltree::Document::parse(text)?;

    let entries = document
        .descendants()
        .filter(|node| node.has_tag_name((ATOM_NAMESPACE, "entry")));
    let releases = entries.filter_map(|entry| {
        let child_text = |name: &str| {
            entry
                .children()
                .find(|node| node.has_tag_name((ATOM_NAMESPACE, name)))
                .and_then(|node| node.text())
        };
        let link = entry
            .children()
            .find(|node| node.has_tag_name((ATOM_NAMESPACE, "link")))
            .and_then(|node| node.attribute("href"));
        let date = child_text("updated").or_else(|| child_text("published"))?;

        Some(FeedRelease {
            version: child_text("title")?.trim().to_owned(),
            day: Day::from_iso(date)?,
            link: link.map(str::to_owned),
            preview: false,
        })
    });
    Ok(releases.collect())
}

/// An RSS 2.0 feed. Items without a title take the last part of their link,
/// so ".../tag/v0.5.0" becomes "v0.5.0".
pub fn parse_rss(body: &[u8]) -> anyhow::Result<Vec<FeedRelease>> {
    let text = std::str::from_utf8(body)?;
    let document = roxmltree::Document::parse(text)?;

    let items = document.descendants().filter(|node| node.has_tag_name("item"));
    let releases = items.filter_map(|item| {
        let child_text = |name: &str| {
            item.children()
                .find(|node| node.has_tag_name(name))
                .and_then(|node| node.text())
                .map(str::trim)
                .filter(|text| !text.is_empty())
        };
        let link = child_text("link");
        let title = child_text("title").or_else(|| link.and_then(last_path_part))?;

        Some(FeedRelease {
            version: title.to_owned(),
            day: Day::from_rfc2822(child_text("pubDate")?)?,
            link: link.map(str::to_owned),
            preview: false,
        })
    });
    Ok(releases.collect())
}

fn last_path_part(link: &str) -> Option<&str> {
    link.trim_end_matches('/').rsplit('/').next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kernel_keeps_mainline_and_stable_only() {
        let body = br#"{"releases":[
            {"moniker":"mainline","version":"7.4-rc1","released":{"isodate":"2026-11-01"}},
            {"moniker":"stable","version":"7.3.3","released":{"isodate":"2026-11-06"}},
            {"moniker":"longterm","version":"6.12.60","released":{"isodate":"2026-11-06"}}
        ]}"#;
        let releases = parse_kernel_org(body).unwrap();

        assert_eq!(releases.len(), 2);
        assert!(releases[0].preview);
        assert_eq!(releases[1].version, "7.3.3");
    }

    #[test]
    fn endoflife_reads_cycles_and_point_releases() {
        let body = br#"{"result":{"releases":[
            {"name":"6.8","releaseDate":"2026-10-14","latest":{"name":"6.8.2","date":"2026-10-27","link":"https://kde.org/announcements/plasma/6/6.8.2/"}}
        ]}}"#;
        let releases = parse_endoflife(body).unwrap();

        assert_eq!(releases.len(), 2);
        assert_eq!(releases[1].version, "6.8.2");
        assert_eq!(releases[1].day, Day { year: 2026, month: 10, day: 27 });
    }

    #[test]
    fn atom_reads_title_date_and_link() {
        let body = br#"<?xml version="1.0"?><feed xmlns="http://www.w3.org/2005/Atom">
            <entry><title>v0.57.1</title><updated>2026-11-03T09:47:53Z</updated>
            <link rel="alternate" href="https://github.com/hyprwm/Hyprland/releases/tag/v0.57.1"/></entry>
        </feed>"#;
        let releases = parse_atom(body).unwrap();

        assert_eq!(releases[0].version, "v0.57.1");
        assert_eq!(releases[0].link.as_deref(), Some("https://github.com/hyprwm/Hyprland/releases/tag/v0.57.1"));
    }

    #[test]
    fn rss_items_without_titles_use_their_link() {
        let body = br#"<rss version="2.0"><channel>
            <item><title/><link>https://codeberg.org/river/river/releases/tag/v0.5.0</link>
            <pubDate>Sat, 10 Oct 2026 10:37:59 +0200</pubDate></item>
        </channel></rss>"#;
        let releases = parse_rss(body).unwrap();

        assert_eq!(releases[0].version, "v0.5.0");
        assert_eq!(releases[0].day, Day { year: 2026, month: 10, day: 10 });
    }
}
