mod catalog_card;
mod collection_card;
mod content;
mod cover_picture;
mod extension_card;
mod fits_together_dialog;
mod idea_card;
mod intro_banner;
mod edge;
mod edge_picture;
mod desktop_scene;
mod icon_tile;
mod layer_stack;
mod protocol_browser;
mod protocol_card;
mod release_card;
mod rice_card;
mod run_check_dialog;
mod logo_tile;
mod mini_desktop;
mod natural_width_cap;
mod screenshot_card;
mod screenshot_gallery;
mod screenshot_picture;
mod showcase_card;
mod spotlight_card;
mod wall;
mod wallpaper_card;
mod wallpaper_dialog;
mod wallpaper_spotlight;
mod what_is_lookbook_dialog;

pub use catalog_card::CatalogCard;
pub use collection_card::CollectionCard;
pub use content::{
    icon_tile, logo_tile, scene_for_desktop, CardBadge, CardContent, CollectionContent, DetailContent, Fact, FeatureCheck,
    ProtocolSummary, ShowcaseContent, SpotlightContent,
};
pub use cover_picture::CoverPicture;
pub use extension_card::extension_card;
pub use fits_together_dialog::{desktop_row, layer_block, FitsTogetherDialog};
pub use idea_card::IdeaCard;
pub use intro_banner::IntroBanner;
pub use edge_picture::EdgePicture;
pub use desktop_scene::{DesktopScene, SceneLayout};
pub use icon_tile::IconTile;
pub use layer_stack::{LayerStack, StackLayer};
pub use protocol_browser::ProtocolBrowser;
pub use release_card::release_card;
pub use rice_card::RiceCard;
pub use run_check_dialog::RunCheckDialog;
pub use logo_tile::{LogoTile, TileContent};
pub use mini_desktop::MiniDesktop;
pub use natural_width_cap::NaturalWidthCap;
pub use screenshot_card::ScreenshotCard;
pub use screenshot_gallery::ScreenshotGallery;
pub use screenshot_picture::ScreenshotPicture;
pub use showcase_card::{screenshot_credit, ShowcaseCard};
pub use spotlight_card::SpotlightCard;
pub use wall::fill_wall_grid;
pub use wallpaper_card::WallpaperCard;
pub use wallpaper_dialog::{WallpaperDialog, WallpaperDialogContext};
pub use wallpaper_spotlight::WallpaperSpotlight;
pub use what_is_lookbook_dialog::WhatIsLookbookDialog;

use gtk::prelude::*;

/// A small rounded label, used for facts like "Rolling" or "pacman".
pub fn chip_label(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.add_css_class("chip");
    label
}

/// A small card with a caption above a value, for example "Packages" and "dnf".
/// A fact card. With a tile, the tile sits on the left of the words.
pub fn fact_tile(fact: &Fact) -> gtk::Box {
    let words = fact_words(fact);
    let Some(content) = &fact.tile else {
        words.add_css_class("card");
        words.add_css_class("fact-tile");
        return words;
    };

    let card = gtk::Box::new(gtk::Orientation::Horizontal, 16);
    card.add_css_class("card");
    card.add_css_class("fact-tile");
    let tile = LogoTile::new(content, FACT_TILE_SIZE);
    tile.set_valign(gtk::Align::Center);
    card.append(&tile);
    words.set_hexpand(true);
    words.set_valign(gtk::Align::Center);
    card.append(&words);
    card
}

const FACT_TILE_SIZE: i32 = 48;
const FACT_TEXT_CHARS: i32 = 26;

fn fact_words(fact: &Fact) -> gtk::Box {
    let tile = gtk::Box::new(gtk::Orientation::Vertical, 2);

    let label = gtk::Label::new(Some(&fact.label));
    label.add_css_class("caption-heading");
    label.add_css_class("dim-label");
    label.set_xalign(0.0);

    let value = gtk::Label::new(Some(&fact.value));
    value.add_css_class("title-4");
    value.set_xalign(0.0);
    value.set_wrap(true);
    // Long names like a GPU's wrap, so two cards still share a row.
    value.set_max_width_chars(FACT_TEXT_CHARS);

    tile.append(&label);
    tile.append(&value);

    if let Some(detail) = &fact.detail {
        let detail_label = gtk::Label::new(Some(detail));
        detail_label.add_css_class("dim-label");
        detail_label.set_xalign(0.0);
        detail_label.set_wrap(true);
        detail_label.set_max_width_chars(FACT_TEXT_CHARS);
        tile.append(&detail_label);
    }
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

/// Opens `url` in the user's browser, from the window that holds `widget`.
pub fn open_link(widget: &impl IsA<gtk::Widget>, url: &str) {
    let window = widget.root().and_downcast::<gtk::Window>();
    gtk::UriLauncher::new(url).launch(window.as_ref(), gtk::gio::Cancellable::NONE, |result| {
        if let Err(error) = result {
            tracing::warn!("Could not open a link: {error}");
        }
    });
}

/// The catalog item a gallery card opens, for both kinds of card.
fn card_item(card: &gtk::Widget) -> Option<crate::catalog::ItemRef> {
    if let Some(card) = card.downcast_ref::<CatalogCard>() {
        return card.item();
    }
    card.downcast_ref::<ShowcaseCard>().and_then(ShowcaseCard::item)
}

/// Opens the details page of the card the user activates in `flowbox`.
pub fn open_details_on_activate(flowbox: &gtk::FlowBox) {
    flowbox.connect_child_activated(|flowbox, child| {
        let Some(item) = child.child().as_ref().and_then(card_item) else {
            return;
        };

        let target = item.to_action_target().to_variant();
        if let Err(error) = flowbox.activate_action("win.show-item", Some(&target)) {
            tracing::warn!("Could not open the details of {}: {error}", item.id);
        }
    });
}

/// Adds a card to a grid when the card's own buttons take the clicks, or it
/// has none. The grid's cell stays out of the Tab order, so each card is one
/// Tab stop and Enter presses its button, not a cell that does nothing.
pub fn append_to_grid(flowbox: &gtk::FlowBox, card: &impl IsA<gtk::Widget>) {
    flowbox.append(card);
    if let Some(cell) = card.parent().and_downcast::<gtk::FlowBoxChild>() {
        cell.set_focusable(false);
    }
}

/// The toggle name that shows every family.
pub const ALL_FAMILIES: &str = "all";

/// Fills a distro family filter: All, then one toggle per family. Explore
/// and Distros share it, so the two filters never drift apart.
pub fn add_family_toggles(toggles: &adw::ToggleGroup) {
    toggles.add(adw::Toggle::builder().name(ALL_FAMILIES).label(gettextrs::gettext("All")).build());
    for family in crate::catalog::DistroFamily::ALL {
        toggles.add(adw::Toggle::builder().name(family.id()).label(family.label()).build());
    }
    toggles.set_active_name(Some(ALL_FAMILIES));
}
