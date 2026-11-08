use adw::subclass::prelude::*;
use gtk::glib;
use gtk::prelude::*;

use super::IconTile;
use crate::catalog::ProtocolInfo;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Lookbook/ui/widgets/protocol_card.ui")]
    pub struct ProtocolCard {
        #[template_child]
        pub art: gtk::TemplateChild<IconTile>,
        #[template_child]
        pub name_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub hidden_label: gtk::TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub interface_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub description_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub useful_for_label: gtk::TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProtocolCard {
        const NAME: &'static str = "ProtocolCard";
        type Type = super::ProtocolCard;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            IconTile::ensure_type();
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProtocolCard {}
    impl WidgetImpl for ProtocolCard {}
    impl BinImpl for ProtocolCard {}
}

glib::wrapper! {
    /// One Wayland protocol: its art, name, what it does and what it is useful for.
    pub struct ProtocolCard(ObjectSubclass<imp::ProtocolCard>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ProtocolCard {
    /// `hidden_in_sandbox` marks protocols the compositor offers but hides from Flatpak apps.
    pub fn new(info: &ProtocolInfo, hidden_in_sandbox: bool) -> Self {
        let card: Self = glib::Object::new();
        let imp = card.imp();

        imp.art.set_art(info.category.color(), info.category.icon_name());
        imp.name_label.set_label(&info.name);
        imp.interface_label.set_label(&info.interface);
        imp.interface_label.set_tooltip_text(Some(&info.interface));
        imp.description_label.set_label(&info.description);
        imp.useful_for_label.set_label(&info.useful_for);
        imp.hidden_label.set_visible(hidden_in_sandbox);
        card
    }
}
