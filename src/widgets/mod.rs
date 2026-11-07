mod catalog_card;
mod collection_card;
mod content;
mod logo_tile;
mod screenshot_card;
mod screenshot_mock;
mod spotlight_card;

pub use catalog_card::CatalogCard;
pub use collection_card::CollectionCard;
pub use content::{CardBadge, CardContent, CollectionContent, SpotlightContent};
pub use logo_tile::{LogoTile, TileContent};
pub use screenshot_card::ScreenshotCard;
pub use screenshot_mock::ScreenshotMock;
pub use spotlight_card::SpotlightCard;

use gtk::prelude::*;

/// A small rounded label, used for facts like "Rolling" or "pacman".
pub fn chip_label(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.add_css_class("chip");
    label
}
