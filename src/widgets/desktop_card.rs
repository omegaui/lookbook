//! A card in the Desktops gallery: a small drawing of the desktop on top, then
//! its name, how far along its Wayland session is, its parts and how to extend it.

use std::cell::{Cell, RefCell};

use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::glib;
use gtk::glib::clone;
use gtk::prelude::*;

use super::{chip_label, CardContent, LogoTile, MiniDesktop, NaturalWidthCap};
use crate::catalog::{Desktop, HostedScreenshot, ItemRef, ToolkitFamily};
use crate::http::shared_session;
use crate::system::SystemInfo;
use crate::wallpapers::fetch_texture;

const TILE_SIZE: i32 = 44;
/// The width a card asks for, so three share a row on a wide window.
const SCREENSHOT_NATURAL_WIDTH: i32 = 300;
const SCREENSHOT_PAGE: &str = "screenshot";

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/LinuxAndYou/ui/widgets/desktop_card.ui")]
    pub struct DesktopCard {
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
        pub wayland_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub tagline_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub chips_box: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub extend_label: gtk::TemplateChild<gtk::Label>,
        pub item: RefCell<Option<ItemRef>>,
        pub toolkit_family: Cell<Option<ToolkitFamily>>,
        pub screenshot: RefCell<Option<HostedScreenshot>>,
        pub screenshot_requested: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DesktopCard {
        const NAME: &'static str = "DesktopCard";
        type Type = super::DesktopCard;
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

    impl ObjectImpl for DesktopCard {}

    impl WidgetImpl for DesktopCard {
        /// The screenshot loads the first time the card is on screen.
        fn map(&self) {
            self.parent_map();
            self.obj().load_screenshot();
        }
    }
    impl BinImpl for DesktopCard {}
}

glib::wrapper! {
    pub struct DesktopCard(ObjectSubclass<imp::DesktopCard>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl DesktopCard {
    pub fn new(desktop: &Desktop, system: &SystemInfo) -> Self {
        let card: Self = glib::Object::new();
        card.show_desktop(desktop, system);
        card
    }

    /// The catalog item this card opens.
    pub fn item(&self) -> Option<ItemRef> {
        self.imp().item.borrow().clone()
    }

    /// The toolkit filter compares against this, for example "gtk".
    pub fn toolkit_id(&self) -> Option<&'static str> {
        self.imp().toolkit_family.get().map(ToolkitFamily::id)
    }

    fn show_desktop(&self, desktop: &Desktop, system: &SystemInfo) {
        let imp = self.imp();
        let content = CardContent::for_desktop(desktop, system);

        imp.item.replace(Some(content.item.clone()));
        imp.toolkit_family.set(Some(desktop.toolkit_family));
        imp.art.set_desktop(&desktop.color, &desktop.bars);
        imp.tile.set_tile_size(TILE_SIZE);
        imp.tile.set_content(&content.tile);
        imp.name_label.set_label(&content.name);
        imp.wayland_label.set_label(&desktop.wayland);
        imp.tagline_label.set_label(&content.description);
        self.show_badge(&content);
        self.show_chips(&content.chips);
        self.show_extend_with(&desktop.extend_with);
        imp.screenshot.replace(desktop.screenshot.clone());
    }

    fn load_screenshot(&self) {
        let imp = self.imp();
        if imp.screenshot_requested.replace(true) {
            return;
        }
        let Some(screenshot) = imp.screenshot.borrow().clone() else {
            return;
        };

        glib::spawn_future_local(clone!(
            #[weak(rename_to = card)]
            self,
            async move {
                match fetch_texture(&shared_session(), &screenshot.url).await {
                    Ok(texture) => card.show_screenshot(&texture, &screenshot),
                    Err(error) => tracing::warn!("Could not load a desktop screenshot: {error:#}"),
                }
            }
        ));
    }

    /// Swaps the drawing for the real screenshot, credited in its tooltip.
    fn show_screenshot(&self, texture: &gtk::gdk::Texture, screenshot: &HostedScreenshot) {
        let picture = gtk::Picture::builder()
            .paintable(texture)
            .content_fit(gtk::ContentFit::Cover)
            .can_shrink(true)
            .tooltip_text(screenshot_credit(screenshot))
            .build();
        picture.update_property(&[gtk::accessible::Property::Label(&gettext("Screenshot"))]);

        let stack = &self.imp().art_stack;
        stack.add_named(&NaturalWidthCap::new(&picture, SCREENSHOT_NATURAL_WIDTH), Some(SCREENSHOT_PAGE));
        stack.set_visible_child_name(SCREENSHOT_PAGE);
    }

    fn show_badge(&self, content: &CardContent) {
        let label = &self.imp().badge_label;
        let Some(badge) = content.badge else {
            return;
        };
        label.set_label(&badge.label());
        label.add_css_class(badge.css_class());
        label.set_visible(true);
    }

    fn show_chips(&self, chips: &[String]) {
        for chip in chips {
            self.imp().chips_box.append(&chip_label(chip));
        }
    }

    /// "Extend with **Shell extensions (GJS)**", with the way in bold.
    fn show_extend_with(&self, extend_with: &str) {
        let bold = format!("<b>{}</b>", glib::markup_escape_text(extend_with));
        let markup = gettext("Extend with {way}").replace("{way}", &bold);
        self.imp().extend_label.set_markup(&markup);
    }
}

/// For example "Screenshot by System76 (GPL-3.0)".
pub fn screenshot_credit(screenshot: &HostedScreenshot) -> String {
    gettext("Screenshot by {credit} ({license})")
        .replace("{credit}", &screenshot.credit)
        .replace("{license}", &screenshot.license)
}
