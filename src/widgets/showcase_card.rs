//! A gallery card with art on top, for the Distros and Desktops pages: a real
//! screenshot, loaded live, with a small drawn desktop until it arrives. Below
//! it come the name, a short line, the tagline, chips and one closing line.

use std::cell::RefCell;

use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::glib;
use gtk::glib::clone;
use gtk::prelude::*;

use super::{chip_label, CardBadge, CoverPicture, LogoTile, MiniDesktop, ShowcaseContent};
use crate::catalog::{HostedScreenshot, ItemRef};
use crate::http::shared_session;
use crate::images::fetch_texture_scaled;

const TILE_SIZE: i32 = 44;
/// Matches the drawn placeholder, so the card keeps its height when the image arrives.
const ART_HEIGHT: i32 = 150;
const SCREENSHOT_PAGE: &str = "screenshot";

/// Gallery cards are about 450 pixels wide; twice that stays sharp on HiDPI.
const CARD_IMAGE_WIDTH: i32 = 960;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Lookbook/ui/widgets/showcase_card.ui")]
    pub struct ShowcaseCard {
        #[template_child]
        pub art_stack: gtk::TemplateChild<gtk::Stack>,
        #[template_child]
        pub art: gtk::TemplateChild<MiniDesktop>,
        #[template_child]
        pub tile: gtk::TemplateChild<LogoTile>,
        #[template_child]
        pub name_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub badge_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub subtitle_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub tagline_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub chips_box: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub footer_icon: gtk::TemplateChild<gtk::Image>,
        #[template_child]
        pub footer_label: gtk::TemplateChild<gtk::Label>,
        pub item: RefCell<Option<ItemRef>>,
        pub toolkit_id: RefCell<Option<&'static str>>,
        /// Taken out when it starts loading, so it loads only once.
        pub screenshot: RefCell<Option<HostedScreenshot>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ShowcaseCard {
        const NAME: &'static str = "ShowcaseCard";
        type Type = super::ShowcaseCard;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            LogoTile::ensure_type();
            MiniDesktop::ensure_type();
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ShowcaseCard {}

    impl WidgetImpl for ShowcaseCard {
        /// The screenshot loads the first time the card is on screen.
        fn map(&self) {
            self.parent_map();
            self.obj().load_screenshot();
        }
    }

    impl BinImpl for ShowcaseCard {}
}

glib::wrapper! {
    pub struct ShowcaseCard(ObjectSubclass<imp::ShowcaseCard>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ShowcaseCard {
    pub fn new(content: &ShowcaseContent) -> Self {
        let card: Self = glib::Object::new();
        card.show_content(content);
        card
    }

    /// The catalog item this card opens.
    pub fn item(&self) -> Option<ItemRef> {
        self.imp().item.borrow().clone()
    }

    /// The Desktops page's toolkit filter compares against this, for example "gtk".
    pub fn toolkit_id(&self) -> Option<&'static str> {
        *self.imp().toolkit_id.borrow()
    }

    fn show_content(&self, content: &ShowcaseContent) {
        let imp = self.imp();
        let card = &content.card;

        imp.item.replace(Some(card.item.clone()));
        imp.toolkit_id.replace(content.toolkit_id);
        imp.screenshot.replace(content.screenshot.clone());
        imp.art.set_desktop(&content.color, &content.bars);
        imp.tile.set_tile_size(TILE_SIZE);
        imp.tile.set_content(&card.tile);
        imp.name_label.set_label(&card.name);
        imp.subtitle_label.set_label(&content.subtitle);
        imp.tagline_label.set_label(&card.description);
        imp.footer_icon.set_icon_name(Some(content.footer_icon));
        imp.footer_label.set_markup(&content.footer_markup);
        self.show_badge(card.badge);
        self.show_chips(&card.chips);
    }

    fn load_screenshot(&self) {
        let Some(screenshot) = self.imp().screenshot.take() else {
            return;
        };

        glib::spawn_future_local(clone!(
            #[weak(rename_to = card)]
            self,
            async move {
                match fetch_texture_scaled(&shared_session(), &screenshot.url, CARD_IMAGE_WIDTH).await {
                    Ok(texture) => card.show_screenshot(&texture, &screenshot),
                    Err(error) => tracing::warn!("Could not load a screenshot: {error:#}"),
                }
            }
        ));
    }

    /// Swaps the drawing for the real screenshot, credited in its tooltip.
    fn show_screenshot(&self, texture: &gtk::gdk::Texture, screenshot: &HostedScreenshot) {
        let picture = CoverPicture::new(texture, ART_HEIGHT);
        picture.set_tooltip_text(Some(&screenshot_credit(screenshot)));
        picture.update_property(&[gtk::accessible::Property::Label(&gettext("Screenshot"))]);

        let stack = &self.imp().art_stack;
        stack.add_named(&picture, Some(SCREENSHOT_PAGE));
        stack.set_visible_child_name(SCREENSHOT_PAGE);
    }

    fn show_badge(&self, badge: Option<CardBadge>) {
        let Some(badge) = badge else {
            return;
        };
        let label = &self.imp().badge_label;
        label.set_label(&badge.label());
        label.add_css_class(badge.css_class());
        label.set_visible(true);
    }

    fn show_chips(&self, chips: &[String]) {
        for chip in chips {
            self.imp().chips_box.append(&chip_label(chip));
        }
    }
}

/// For example "Screenshot by System76 (GPL-3.0)".
pub fn screenshot_credit(screenshot: &HostedScreenshot) -> String {
    gettext("Screenshot by {credit} ({license})")
        .replace("{credit}", &screenshot.credit)
        .replace("{license}", &screenshot.license)
}
