use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::glib;
use gtk::prelude::*;

use super::{LogoTile, TileContent};
use crate::catalog::Idea;
use crate::ideas::IdeaStatus;

const TILE_SIZE: i32 = 52;
/// Ideas that work elsewhere get a gray tile, so ready ones stand out.
const ELSEWHERE_TILE_COLOR: &str = "#77767b";

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Syspedia/ui/widgets/idea_card.ui")]
    pub struct IdeaCard {
        #[template_child]
        pub tile: gtk::TemplateChild<LogoTile>,
        #[template_child]
        pub name_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub status_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub description_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub route_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub interface_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub places_label: gtk::TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IdeaCard {
        const NAME: &'static str = "IdeaCard";
        type Type = super::IdeaCard;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            LogoTile::ensure_type();
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
        card.show_idea(idea, status.is_available());
        card.show_status(status);
        card
    }

    fn show_idea(&self, idea: &Idea, is_available: bool) {
        let imp = self.imp();
        let tile_color = if is_available { idea.color.as_str() } else { ELSEWHERE_TILE_COLOR };

        imp.tile.set_tile_size(TILE_SIZE);
        imp.tile.set_content(&TileContent {
            short: String::new(),
            color: tile_color.to_owned(),
            logo: None,
            icon_name: Some(idea.icon.clone()),
        });
        imp.name_label.set_label(&idea.name);
        imp.description_label.set_label(&idea.description);
    }

    fn show_status(&self, status: &IdeaStatus) {
        match status {
            IdeaStatus::BuiltIn { by, route } => self.show_built_in(by, route.as_deref()),
            IdeaStatus::Ready { route, interface } => self.show_ready(route, interface.as_deref()),
            IdeaStatus::WorksElsewhere { needs, places } => self.show_elsewhere(needs, places),
        }
    }

    /// "Built in", and how to make your own instead when that is possible too.
    fn show_built_in(&self, by: &str, route: Option<&str>) {
        let imp = self.imp();
        imp.status_label.set_label(&gettext("Built in"));
        imp.status_label.add_css_class("ready");
        let text = match route {
            Some(route) => gettext("Part of {by}. Make your own with: {route}")
                .replace("{by}", by)
                .replace("{route}", route),
            None => gettext("Part of {}").replace("{}", by),
        };
        imp.route_label.set_label(&text);
    }

    fn show_ready(&self, route: &str, interface: Option<&str>) {
        let imp = self.imp();
        imp.status_label.set_label(&gettext("Ready here"));
        imp.status_label.add_css_class("ready");
        imp.route_label.set_label(route);

        if let Some(interface) = interface {
            imp.interface_label.set_label(interface);
            imp.interface_label.set_tooltip_text(Some(interface));
            imp.interface_label.set_visible(true);
        }
    }

    fn show_elsewhere(&self, needs: &str, places: &[String]) {
        let imp = self.imp();
        self.add_css_class("elsewhere");
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
