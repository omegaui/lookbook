mod catalog_card;
mod collection_card;
mod content;
mod idea_card;
mod iso_scene;
mod logo_tile;
mod screenshot_card;
mod screenshot_mock;
mod spotlight_card;
mod wall;
mod wallpaper_card;
mod wallpaper_dialog;

pub use catalog_card::CatalogCard;
pub use collection_card::CollectionCard;
pub use content::{
    CardBadge, CardContent, CollectionContent, DetailContent, Fact, FeatureCheck, ProtocolSummary,
    SpotlightContent,
};
pub use idea_card::IdeaCard;
pub use iso_scene::{IsoScene, SceneLayout};
pub use logo_tile::{LogoTile, TileContent};
pub use screenshot_card::ScreenshotCard;
pub use screenshot_mock::ScreenshotMock;
pub use spotlight_card::SpotlightCard;
pub use wall::fill_wall_grid;
pub use wallpaper_card::WallpaperCard;
pub use wallpaper_dialog::{WallpaperDialog, WallpaperDialogContext};

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

/// A check mark or a cross next to a feature name, for example "✓ Layer shell".
pub fn feature_check(check: &FeatureCheck) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    row.add_css_class("feature-check");

    let (icon_name, css_class) = if check.supported {
        ("object-select-symbolic", "supported")
    } else {
        ("window-close-symbolic", "missing")
    };
    row.add_css_class(css_class);

    let icon = gtk::Image::from_icon_name(icon_name);
    icon.set_pixel_size(14);
    icon.set_accessible_role(gtk::AccessibleRole::Presentation);

    let label = gtk::Label::new(Some(&check.label));
    label.add_css_class("caption-heading");

    let state = if check.supported {
        gettextrs::gettext("Supported")
    } else {
        gettextrs::gettext("Not supported")
    };
    let description = match &check.note {
        Some(note) => format!("{state}. {note}"),
        None => state,
    };
    row.update_property(&[gtk::accessible::Property::Description(&description)]);
    row.set_tooltip_text(check.note.as_deref());

    row.append(&icon);
    row.append(&label);
    row
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
