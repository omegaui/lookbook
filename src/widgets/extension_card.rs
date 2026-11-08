//! A GNOME Shell extension from extensions.gnome.org: its screenshot first,
//! then its name, creator and downloads. Clicking it opens the extension's page.

use gettextrs::ngettext;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{gio, glib};

use super::NaturalWidthCap;
use crate::extensions::{compact_count, Extension};
use crate::wallpapers::fetch_texture;

const SCREENSHOT_HEIGHT: i32 = 150;
/// The width a card asks for, so three share a row on a wide window.
const CARD_NATURAL_WIDTH: i32 = 320;

pub fn extension_card(extension: &Extension, session: &soup::Session) -> gtk::Button {
    let picture = gtk::Picture::builder()
        .height_request(SCREENSHOT_HEIGHT)
        .content_fit(gtk::ContentFit::Cover)
        .can_shrink(true)
        .overflow(gtk::Overflow::Hidden)
        .css_classes(["wallpaper-thumbnail"])
        .build();

    let name = gtk::Label::builder()
        .label(&extension.name)
        .xalign(0.0)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .css_classes(["heading"])
        .build();

    let byline = gtk::Label::builder()
        .label(
            ngettext(
                "by {creator} · {downloads} download",
                "by {creator} · {downloads} downloads",
                plural_count(extension.downloads),
            )
            .replace("{creator}", &extension.creator.display_name)
                .replace("{downloads}", &compact_count(extension.downloads)),
        )
        .xalign(0.0)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .css_classes(["caption", "dim-label"])
        .build();

    // A puzzle piece stands in until the screenshot arrives, or for good if there is none.
    let placeholder = gtk::Image::builder()
        .icon_name("application-x-addon-symbolic")
        .pixel_size(48)
        .css_classes(["dim-label"])
        .accessible_role(gtk::AccessibleRole::Presentation)
        .build();
    let screenshot = gtk::Overlay::builder().child(&picture).build();
    screenshot.add_overlay(&placeholder);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    content.append(&screenshot);
    content.append(&name);
    content.append(&byline);

    let button = gtk::Button::builder()
        .child(&NaturalWidthCap::new(&content, CARD_NATURAL_WIDTH))
        .css_classes(["flat", "wallpaper-card"])
        .tooltip_text(&extension.description)
        .build();

    let page_url = extension.page_url();
    button.connect_clicked(move |button| open_page(button, &page_url));

    if let Some(url) = extension.screenshot.clone() {
        load_screenshot(&picture, &placeholder, session, url);
    }
    button
}

fn load_screenshot(picture: &gtk::Picture, placeholder: &gtk::Image, session: &soup::Session, url: String) {
    glib::spawn_future_local(clone!(
        #[weak]
        picture,
        #[weak]
        placeholder,
        #[strong]
        session,
        async move {
            match fetch_texture(&session, &url).await {
                Ok(texture) => {
                    picture.set_paintable(Some(&texture));
                    placeholder.set_visible(false);
                }
                Err(error) => tracing::warn!("Could not load an extension screenshot: {error:#}"),
            }
        }
    ));
}

fn open_page(button: &gtk::Button, url: &str) {
    let window = button.root().and_downcast::<gtk::Window>();
    gtk::UriLauncher::new(url).launch(window.as_ref(), gio::Cancellable::NONE, |result| {
        if let Err(error) = result {
            tracing::warn!("Could not open the extension page: {error}");
        }
    });
}

/// Plural rules need a whole number; counts this large all use the plural.
fn plural_count(count: u64) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}
