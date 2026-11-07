//! The details page of one distro, desktop or compositor.

use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use gettextrs::gettext;

use crate::catalog::{Catalog, Screenshot};
use crate::widgets::{
    fact_tile, feature_check, fill_protocol_lists, fill_wall_grid, CardBadge, DetailContent,
    LogoTile, ProtocolSummary,
};

const TILE_SIZE: i32 = 96;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/LinuxAndYou/ui/pages/item_detail_page.ui")]
    pub struct ItemDetailPage {
        #[template_child]
        pub tile: gtk::TemplateChild<LogoTile>,
        #[template_child]
        pub name_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub badge_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub tagline_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub wall_section: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub wall_grid: gtk::TemplateChild<gtk::Grid>,
        #[template_child]
        pub facts_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub protocols_section: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub protocols_summary_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub checks_box: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub protocol_cards_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub more_protocols_expander: gtk::TemplateChild<gtk::Expander>,
        #[template_child]
        pub more_protocols_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        pub website: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ItemDetailPage {
        const NAME: &'static str = "ItemDetailPage";
        type Type = super::ItemDetailPage;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            LogoTile::ensure_type();
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ItemDetailPage {}
    impl WidgetImpl for ItemDetailPage {}
    impl NavigationPageImpl for ItemDetailPage {}

    #[gtk::template_callbacks]
    impl ItemDetailPage {
        #[template_callback]
        fn on_website_clicked(&self, _button: &gtk::Button) {
            self.obj().open_website();
        }
    }
}

glib::wrapper! {
    pub struct ItemDetailPage(ObjectSubclass<imp::ItemDetailPage>)
        @extends gtk::Widget, adw::NavigationPage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ItemDetailPage {
    pub fn new(content: &DetailContent, screenshots: &[&Screenshot], catalog: &Catalog) -> Self {
        let page: Self = glib::Object::builder()
            .property("title", &content.title)
            .build();

        page.show_header(content);
        page.show_screenshots(screenshots);
        page.show_facts(content);
        page.show_protocols(content, catalog);
        page
    }

    fn show_header(&self, content: &DetailContent) {
        let imp = self.imp();
        imp.tile.set_tile_size(TILE_SIZE);
        imp.tile.set_content(&content.tile);
        imp.name_label.set_label(&content.title);
        imp.tagline_label.set_label(&content.tagline);
        imp.website.replace(content.website.clone());
        self.show_badge(content.badge);
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

    /// The Hall of fame section stays hidden when there are no screenshots yet.
    fn show_screenshots(&self, screenshots: &[&Screenshot]) {
        let imp = self.imp();
        fill_wall_grid(&imp.wall_grid, screenshots);
        imp.wall_section.set_visible(!screenshots.is_empty());
    }

    fn show_facts(&self, content: &DetailContent) {
        for fact in &content.facts {
            self.imp().facts_flowbox.append(&fact_tile(fact));
        }
    }

    /// Shows the key feature checks and what the compositor offers: a card per
    /// described protocol, and the rest under "more protocols".
    fn show_protocols(&self, content: &DetailContent, catalog: &Catalog) {
        let Some(protocols) = &content.protocols else {
            return;
        };
        let imp = self.imp();
        let overview = catalog.protocol_overview(&protocols.globals);

        imp.protocols_summary_label
            .set_label(&protocols_summary_text(protocols));

        for check in &content.checks {
            imp.checks_box.append(&feature_check(check));
        }

        fill_protocol_lists(
            &imp.protocol_cards_flowbox,
            &imp.more_protocols_flowbox,
            &overview,
            None,
        );
        imp.more_protocols_expander.set_label(Some(
            &gettext("{} more protocols").replace("{}", &overview.others.len().to_string()),
        ));
        imp.more_protocols_expander
            .set_visible(!overview.others.is_empty());
        imp.protocols_section.set_visible(true);
    }

    fn open_website(&self) {
        let website = self.imp().website.borrow().clone();
        let window = self.root().and_downcast::<gtk::Window>();

        gtk::UriLauncher::new(&website).launch(window.as_ref(), gio::Cancellable::NONE, |result| {
            if let Err(error) = result {
                tracing::warn!("Could not open the website: {error}");
            }
        });
    }
}

/// For example "63 protocols in version 0.52.1, checked on 2025-11-20. Data from wayland-explorer."
fn protocols_summary_text(protocols: &ProtocolSummary) -> String {
    gettext("{count} protocols in version {version}, checked on {date}. Data from wayland-explorer.")
        .replace("{count}", &protocols.globals.len().to_string())
        .replace("{version}", &protocols.version)
        .replace("{date}", &protocols.checked_on)
}
