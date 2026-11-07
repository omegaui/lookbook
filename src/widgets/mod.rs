mod catalog_card;
mod collection_card;
mod content;
mod logo_tile;
mod screenshot_card;
mod screenshot_mock;
mod spotlight_card;
mod wall;

pub use catalog_card::CatalogCard;
pub use collection_card::CollectionCard;
pub use content::{CardBadge, CardContent, CollectionContent, DetailContent, Fact, SpotlightContent};
pub use logo_tile::{LogoTile, TileContent};
pub use screenshot_card::ScreenshotCard;
pub use screenshot_mock::ScreenshotMock;
pub use spotlight_card::SpotlightCard;
pub use wall::fill_wall_grid;

use gtk::prelude::*;

/// A small rounded label, used for facts like "Rolling" or "pacman".
pub fn chip_label(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.add_css_class("chip");
    label
}

/// A small card with a caption above a value, for example "Packages" and "dnf".
pub fn fact_tile(fact: &Fact) -> gtk::Box {
    let tile = gtk::Box::new(gtk::Orientation::Vertical, 2);
    tile.add_css_class("card");
    tile.add_css_class("fact-tile");

    let label = gtk::Label::new(Some(&fact.label));
    label.add_css_class("caption-heading");
    label.add_css_class("dim-label");
    label.set_xalign(0.0);

    let value = gtk::Label::new(Some(&fact.value));
    value.add_css_class("title-4");
    value.set_xalign(0.0);
    value.set_wrap(true);

    tile.append(&label);
    tile.append(&value);
    tile
}

/// Opens the details page of the card the user activates in `flowbox`.
pub fn open_details_on_activate(flowbox: &gtk::FlowBox) {
    flowbox.connect_child_activated(|flowbox, child| {
        let Some(card) = child.child().and_downcast::<CatalogCard>() else {
            return;
        };
        let Some(item) = card.item() else {
            return;
        };

        let target = item.to_action_target().to_variant();
        if let Err(error) = flowbox.activate_action("win.show-item", Some(&target)) {
            tracing::warn!("Could not open the details of {}: {error}", item.id);
        }
    });
}
