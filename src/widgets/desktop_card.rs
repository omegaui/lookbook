//! A card in the Desktops gallery: a small drawing of the desktop on top, then
//! its name, how far along its Wayland session is, its parts and how to extend it.

use std::cell::{Cell, RefCell};

use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::glib;
use gtk::prelude::*;

use super::{chip_label, CardContent, LogoTile, MiniDesktop};
use crate::catalog::{Desktop, ItemRef, ToolkitFamily};
use crate::system::SystemInfo;

const TILE_SIZE: i32 = 44;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/LinuxAndYou/ui/widgets/desktop_card.ui")]
    pub struct DesktopCard {
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
    impl WidgetImpl for DesktopCard {}
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
