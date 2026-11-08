use std::cell::RefCell;

use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::glib;

use super::{chip_label, feature_check, CardBadge, CardContent, FeatureCheck, LogoTile};
use crate::catalog::ItemRef;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Lookbook/ui/widgets/catalog_card.ui")]
    pub struct CatalogCard {
        #[template_child]
        pub tile: gtk::TemplateChild<LogoTile>,
        #[template_child]
        pub name_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub badge_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub description_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub chips_box: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub checks_box: gtk::TemplateChild<gtk::Box>,
        pub item: RefCell<Option<ItemRef>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CatalogCard {
        const NAME: &'static str = "CatalogCard";
        type Type = super::CatalogCard;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            LogoTile::ensure_type();
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CatalogCard {}
    impl WidgetImpl for CatalogCard {}
    impl BinImpl for CatalogCard {}
}

glib::wrapper! {
    /// A card for one distro, desktop or compositor.
    pub struct CatalogCard(ObjectSubclass<imp::CatalogCard>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl CatalogCard {
    pub fn new(content: &CardContent) -> Self {
        let card: Self = glib::Object::new();
        card.show_content(content);
        card
    }

    /// The catalog item this card opens.
    pub fn item(&self) -> Option<ItemRef> {
        self.imp().item.borrow().clone()
    }

    fn show_content(&self, content: &CardContent) {
        let imp = self.imp();
        imp.item.replace(Some(content.item.clone()));
        imp.tile.set_content(&content.tile);
        imp.name_label.set_label(&content.name);
        imp.description_label.set_label(&content.description);
        self.show_badge(content.badge);
        self.show_chips(&content.chips);
        self.show_checks(&content.checks);
    }

    fn show_checks(&self, checks: &[FeatureCheck]) {
        let checks_box = &self.imp().checks_box;
        for check in checks {
            checks_box.append(&feature_check(check));
        }
        checks_box.set_visible(!checks.is_empty());
    }

    fn show_badge(&self, badge: Option<CardBadge>) {
        CardBadge::show_on(badge, &self.imp().badge_label);
    }

    fn show_chips(&self, chips: &[String]) {
        for chip in chips {
            self.imp().chips_box.append(&chip_label(chip));
        }
    }
}
