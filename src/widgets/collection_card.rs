use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;

use super::{CollectionContent, LogoTile};

const TILE_SIZE: i32 = 48;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Syspedia/ui/widgets/collection_card.ui")]
    pub struct CollectionCard {
        #[template_child]
        pub tiles_box: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub title_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub names_label: gtk::TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CollectionCard {
        const NAME: &'static str = "CollectionCard";
        type Type = super::CollectionCard;
        type ParentType = gtk::Button;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CollectionCard {}
    impl WidgetImpl for CollectionCard {}
    impl ButtonImpl for CollectionCard {}
}

glib::wrapper! {
    /// One of the cards at the top of Explore, for example "Rolling releases".
    pub struct CollectionCard(ObjectSubclass<imp::CollectionCard>)
        @extends gtk::Widget, gtk::Button,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl CollectionCard {
    pub fn new(content: &CollectionContent) -> Self {
        let card: Self = glib::Object::new();
        card.show_content(content);
        card
    }

    fn show_content(&self, content: &CollectionContent) {
        let imp = self.imp();
        self.add_css_class(content.accent.css_class());
        imp.title_label.set_label(&content.title);
        imp.names_label.set_label(&content.names);
        self.set_action_target_value(Some(&content.page.to_variant()));
        self.update_property(&[gtk::accessible::Property::Label(&content.title)]);

        for tile in &content.tiles {
            imp.tiles_box.append(&LogoTile::new(tile, TILE_SIZE));
        }
    }
}
