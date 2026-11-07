use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;

use super::{chip_label, LogoTile, SpotlightContent};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/LinuxAndYou/ui/widgets/spotlight_card.ui")]
    pub struct SpotlightCard {
        #[template_child]
        pub tile: gtk::TemplateChild<LogoTile>,
        #[template_child]
        pub kicker_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub name_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub tagline_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub chips_box: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub details_button: gtk::TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SpotlightCard {
        const NAME: &'static str = "SpotlightCard";
        type Type = super::SpotlightCard;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            LogoTile::ensure_type();
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SpotlightCard {}
    impl WidgetImpl for SpotlightCard {}
    impl BoxImpl for SpotlightCard {}
}

glib::wrapper! {
    /// A page of the Explore carousel that features one item.
    pub struct SpotlightCard(ObjectSubclass<imp::SpotlightCard>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl SpotlightCard {
    pub fn new(content: &SpotlightContent) -> Self {
        let card: Self = glib::Object::new();
        card.show_content(content);
        card
    }

    fn show_content(&self, content: &SpotlightContent) {
        let imp = self.imp();
        self.add_css_class(content.accent.css_class());
        imp.tile.set_content(&content.tile);
        imp.kicker_label.set_label(&content.kicker);
        imp.name_label.set_label(&content.name);
        imp.tagline_label.set_label(&content.tagline);
        imp.details_button
            .set_action_target_value(Some(&content.page_name.to_variant()));

        for chip in &content.chips {
            imp.chips_box.append(&chip_label(chip));
        }
    }
}
