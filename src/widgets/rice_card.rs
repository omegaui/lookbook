//! A card for one rice: its own picture or clip on top, then its name, author,
//! what it is built with, the compositors it runs on, and a Get It button that
//! opens its repository.
//!
//! Pictures load the first time the card is on screen. Clips are bigger, so
//! they wait for the Play button; then the whole clip downloads into memory
//! (so it can loop) and plays muted.

use std::cell::RefCell;

use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{gio, glib};

use super::{chip_label, open_link, CoverPicture, LogoTile, TileContent};
use crate::catalog::{Catalog, Rice, RicePreview, RicePreviewKind};
use crate::http::shared_session;
use crate::images::fetch_texture_scaled;
use crate::wallpapers::fetch_bytes;

/// The same height as the placeholder, so the card keeps its size.
const PREVIEW_HEIGHT: i32 = 190;
const PREVIEW_PAGE: &str = "preview";
const COMPOSITOR_TILE_SIZE: i32 = 22;

/// Rice cards are about 380 pixels wide; README pictures are often 4K.
const PREVIEW_IMAGE_WIDTH: i32 = 760;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Lookbook/ui/widgets/rice_card.ui")]
    pub struct RiceCard {
        #[template_child]
        pub preview_stack: gtk::TemplateChild<gtk::Stack>,
        #[template_child]
        pub spinner: gtk::TemplateChild<adw::Spinner>,
        #[template_child]
        pub play_button: gtk::TemplateChild<gtk::Button>,
        #[template_child]
        pub name_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub author_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub tagline_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub chips_box: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub compositors_box: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub details_label: gtk::TemplateChild<gtk::Label>,
        pub link: RefCell<String>,
        /// Taken out when it starts loading, so it loads only once.
        pub preview: RefCell<Option<RicePreview>>,
        /// The playing clip, paused while the card is scrolled out of view.
        pub clip: RefCell<Option<gtk::MediaFile>>,
        /// The clip's address, kept so Play can bring it back after the page
        /// was left and the clip freed.
        pub clip_preview: RefCell<Option<RicePreview>>,
        /// Watches the page's scrolling while the card is on screen.
        pub scroll_watch: RefCell<Option<(gtk::Adjustment, glib::SignalHandlerId)>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RiceCard {
        const NAME: &'static str = "RiceCard";
        type Type = super::RiceCard;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RiceCard {}

    impl WidgetImpl for RiceCard {
        /// Pictures load the first time the card is on screen.
        fn map(&self) {
            self.parent_map();
            let card = self.obj();
            card.load_picture();
            card.watch_scrolling();
        }

        /// Leaving the page frees the clip; Play loads it again.
        fn unmap(&self) {
            let card = self.obj();
            card.stop_watching_scrolling();
            card.release_clip();
            self.parent_unmap();
        }
    }

    impl BinImpl for RiceCard {}

    #[gtk::template_callbacks]
    impl RiceCard {
        #[template_callback]
        fn on_play_clicked(&self) {
            self.obj().play_clip();
        }

        #[template_callback]
        fn on_get_clicked(&self) {
            let card = self.obj();
            let link = self.link.borrow().clone();
            open_link(&*card, &link);
        }
    }
}

glib::wrapper! {
    pub struct RiceCard(ObjectSubclass<imp::RiceCard>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl RiceCard {
    pub fn new(rice: &Rice, catalog: &Catalog) -> Self {
        let card: Self = glib::Object::new();
        card.show_rice(rice, catalog);
        card
    }

    fn show_rice(&self, rice: &Rice, catalog: &Catalog) {
        let imp = self.imp();
        imp.name_label.set_label(&rice.name);
        imp.author_label
            .set_label(&gettext("by {}").replace("{}", &rice.author));
        imp.tagline_label.set_label(&rice.tagline);
        imp.details_label.set_label(&details_text(rice));
        imp.link.replace(rice.link.clone());
        imp.preview.replace(Some(rice.preview.clone()));
        imp.play_button
            .set_visible(rice.preview.kind == RicePreviewKind::Video);

        for tool in &rice.built_with {
            imp.chips_box.append(&chip_label(tool));
        }
        self.show_compositors(rice, catalog);
    }

    /// A small logo per compositor, named in its tooltip.
    fn show_compositors(&self, rice: &Rice, catalog: &Catalog) {
        for compositor in rice.compositors.iter().filter_map(|id| catalog.find_compositor(id)) {
            let tile = LogoTile::labelled(
                &TileContent {
                    short: compositor.short.clone(),
                    color: compositor.color.clone(),
                    logo: compositor.logo.clone(),
                    icon_name: None,
                },
                COMPOSITOR_TILE_SIZE,
                &compositor.name,
            );
            tile.set_tooltip_text(Some(&compositor.name));
            self.imp().compositors_box.append(&tile);
        }
    }

    fn load_picture(&self) {
        let imp = self.imp();
        let is_picture = imp
            .preview
            .borrow()
            .as_ref()
            .is_some_and(|preview| preview.kind == RicePreviewKind::Image);
        if !is_picture {
            return;
        }
        let Some(preview) = imp.preview.take() else {
            return;
        };

        glib::spawn_future_local(clone!(
            #[weak(rename_to = card)]
            self,
            async move {
                match fetch_texture_scaled(&shared_session(), &preview.url, PREVIEW_IMAGE_WIDTH).await {
                    Ok(texture) => card.show_preview(&texture),
                    Err(error) => tracing::warn!("Could not load a rice picture: {error:#}"),
                }
            }
        ));
    }

    /// Downloads the whole clip, then plays it muted and on a loop.
    fn play_clip(&self) {
        let imp = self.imp();
        let Some(preview) = imp.preview.take() else {
            return;
        };
        imp.clip_preview.replace(Some(preview.clone()));
        imp.play_button.set_visible(false);
        imp.spinner.set_visible(true);

        glib::spawn_future_local(clone!(
            #[weak(rename_to = card)]
            self,
            async move {
                let result = fetch_bytes(&shared_session(), &preview.url).await;
                card.imp().spinner.set_visible(false);
                match result {
                    Ok(bytes) => card.show_clip(&bytes),
                    Err(error) => {
                        tracing::warn!("Could not load a rice clip: {error:#}");
                        card.imp().preview.replace(Some(preview));
                        card.imp().play_button.set_visible(true);
                    }
                }
            }
        ));
    }

    fn show_clip(&self, bytes: &glib::Bytes) {
        let stream = gio::MemoryInputStream::from_bytes(bytes);
        let media = gtk::MediaFile::for_input_stream(&stream);
        media.set_muted(true);
        media.set_loop(true);
        media.play();
        self.show_preview(&media);
        self.imp().clip.replace(Some(media));
    }

    /// Plays the clip only while the card is in view.
    fn watch_scrolling(&self) {
        let Some(scrolled) = self.ancestor(gtk::ScrolledWindow::static_type()).and_downcast::<gtk::ScrolledWindow>() else {
            return;
        };
        let adjustment = scrolled.vadjustment();
        let handler = adjustment.connect_value_changed(clone!(
            #[weak(rename_to = card)]
            self,
            move |_| card.play_when_in_view()
        ));
        self.imp().scroll_watch.replace(Some((adjustment, handler)));
        self.play_when_in_view();
    }

    fn stop_watching_scrolling(&self) {
        if let Some((adjustment, handler)) = self.imp().scroll_watch.take() {
            adjustment.disconnect(handler);
        }
    }

    fn play_when_in_view(&self) {
        let Some(clip) = self.imp().clip.borrow().clone() else {
            return;
        };
        if self.is_in_view() {
            clip.play();
        } else {
            clip.pause();
        }
    }

    fn is_in_view(&self) -> bool {
        let Some(scrolled) = self.ancestor(gtk::ScrolledWindow::static_type()) else {
            return true;
        };
        let Some(bounds) = self.compute_bounds(&scrolled) else {
            return false;
        };
        bounds.y() + bounds.height() > 0.0 && bounds.y() < scrolled.height() as f32
    }

    /// Drops the decoded clip and its bytes, and shows Play again.
    fn release_clip(&self) {
        let imp = self.imp();
        let Some(clip) = imp.clip.take() else {
            return;
        };
        clip.pause();
        if let Some(stale) = imp.preview_stack.child_by_name(PREVIEW_PAGE) {
            imp.preview_stack.remove(&stale);
        }
        imp.preview_stack.set_visible_child_name("placeholder");
        imp.preview.replace(imp.clip_preview.take());
        imp.play_button.set_visible(true);
    }

    fn show_preview(&self, paintable: &impl IsA<gtk::gdk::Paintable>) {
        let picture = CoverPicture::new(paintable, PREVIEW_HEIGHT);
        let name = self.imp().name_label.label();
        picture.update_property(&[gtk::accessible::Property::Label(
            &gettext("Preview of {}").replace("{}", &name),
        )]);
        let stack = &self.imp().preview_stack;
        stack.add_named(&picture, Some(PREVIEW_PAGE));
        stack.set_visible_child_name(PREVIEW_PAGE);
    }
}

/// For example "Installs on Arch, Fedora · GPL-3.0".
fn details_text(rice: &Rice) -> String {
    match &rice.installs_on {
        Some(distros) => gettext("Installs on {distros} · {license}")
            .replace("{distros}", distros)
            .replace("{license}", &rice.license),
        None => rice.license.clone(),
    }
}
