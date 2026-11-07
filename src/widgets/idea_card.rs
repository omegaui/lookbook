use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::glib;
use gtk::prelude::*;

use crate::catalog::Idea;
use crate::ideas::IdeaStatus;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/LinuxAndYou/ui/widgets/idea_card.ui")]
    pub struct IdeaCard {
        #[template_child]
        pub status_icon: gtk::TemplateChild<gtk::Image>,
        #[template_child]
        pub name_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub status_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub description_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub route_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub places_label: gtk::TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IdeaCard {
        const NAME: &'static str = "IdeaCard";
        type Type = super::IdeaCard;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for IdeaCard {}
    impl WidgetImpl for IdeaCard {}
    impl BinImpl for IdeaCard {}
}

glib::wrapper! {
    /// One idea on the Your System page, with whether it is ready on this system.
    pub struct IdeaCard(ObjectSubclass<imp::IdeaCard>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl IdeaCard {
    pub fn new(idea: &Idea, status: &IdeaStatus) -> Self {
        let card: Self = glib::Object::new();
        card.show_idea(idea);
        card.show_status(status);
        card
    }

    fn show_idea(&self, idea: &Idea) {
        let imp = self.imp();
        imp.name_label.set_label(&idea.name);
        imp.description_label.set_label(&idea.description);
    }

    fn show_status(&self, status: &IdeaStatus) {
        match status {
            IdeaStatus::Ready { route } => self.show_ready(route),
            IdeaStatus::WorksElsewhere { needs, places } => self.show_elsewhere(needs, places),
        }
    }

    fn show_ready(&self, route: &str) {
        let imp = self.imp();
        imp.status_icon.set_icon_name(Some("object-select-symbolic"));
        imp.status_label.set_label(&gettext("Ready here"));
        imp.status_label.add_css_class("ready");
        imp.route_label.set_label(route);
    }

    fn show_elsewhere(&self, needs: &str, places: &[String]) {
        let imp = self.imp();
        self.add_css_class("elsewhere");
        imp.status_icon.set_icon_name(Some("go-next-symbolic"));
        imp.status_label.set_label(&gettext("Works elsewhere"));
        imp.status_label.add_css_class("elsewhere");
        imp.route_label
            .set_label(&gettext("Needs {}").replace("{}", needs));

        if !places.is_empty() {
            imp.places_label
                .set_label(&gettext("Works on {}").replace("{}", &places.join(", ")));
            imp.places_label.set_visible(true);
        }
    }
}
