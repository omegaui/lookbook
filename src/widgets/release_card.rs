//! A card for one recent release: the project's tile, its name and version,
//! when it came out and what kind of project it is. Clicking it opens the
//! release notes or the project's page.

use gettextrs::gettext;
use gtk::glib;
use gtk::prelude::*;

use super::{open_link, LogoTile, TileContent};
use crate::catalog::NewsCategory;
use crate::news::{Day, Release};

const TILE_SIZE: i32 = 44;

pub fn release_card(release: &Release) -> gtk::Button {
    let tile = LogoTile::new(&tile_content(release), TILE_SIZE);
    tile.set_valign(gtk::Align::Center);

    let name = label(&release.source.name, &["caption-heading", "dim-label"]);
    let version = label(&release.version, &["title-4"]);
    version.set_ellipsize(gtk::pango::EllipsizeMode::End);
    let details = label(&details_text(release), &["caption", "dim-label"]);

    let text = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text.set_valign(gtk::Align::Center);
    text.set_hexpand(true);
    text.append(&name);
    text.append(&version);
    text.append(&details);

    let content = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    content.append(&tile);
    content.append(&text);

    let button = gtk::Button::builder()
        .child(&content)
        .css_classes(["card", "release-card"])
        .tooltip_text(gettext("Read the release notes in your browser"))
        .build();
    let link = release.link.clone();
    button.connect_clicked(move |button| open_link(button, &link));
    button
}

fn tile_content(release: &Release) -> TileContent {
    TileContent {
        short: release.source.short.clone(),
        color: release.source.color.clone(),
        logo: release.source.logo.clone(),
        icon_name: None,
    }
}

fn label(text: &str, css_classes: &[&str]) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.0);
    for css_class in css_classes {
        label.add_css_class(css_class);
    }
    label
}

/// For example "Compositor · Nov 3", or "Kernel · Preview · Nov 1".
fn details_text(release: &Release) -> String {
    let mut parts = vec![category_label(release.source.category)];
    if release.preview {
        parts.push(gettext("Preview"));
    }
    parts.push(short_date(release.day));
    parts.join(" · ")
}

fn category_label(category: NewsCategory) -> String {
    match category {
        NewsCategory::Kernel => gettext("Kernel"),
        NewsCategory::Distro => gettext("Distro"),
        NewsCategory::Desktop => gettext("Desktop"),
        NewsCategory::Compositor => gettext("Compositor"),
    }
}

/// "Nov 3" in the user's language.
fn short_date(day: Day) -> String {
    glib::DateTime::from_utc(day.year, day.month as i32, day.day as i32, 0, 0, 0.0)
        // Translators: a short date in strftime format, for example "Nov 3".
        // Reorder it as your language writes dates, for example "%-d %b".
        .and_then(|date| date.format(&gettext("%b %-d")))
        .map(|text| text.to_string())
        .unwrap_or_default()
}
