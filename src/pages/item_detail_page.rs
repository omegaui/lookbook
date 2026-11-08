//! The details page of one distro, desktop or compositor.

use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::glib::clone;
use gtk::glib;

use crate::images::load_texture_for;
use crate::catalog::{Catalog, HostedScreenshot, ItemKind, ItemRef};
use crate::details::{compare, setup_for, similar_items, Comparison};
use crate::extensions::fetch_popular;
use crate::ideas::evaluate;
use crate::system::SystemInfo;
use crate::http::shared_session;
use crate::widgets::{
    append_to_grid, open_link,
    chip_label, extension_card, fact_tile, feature_check,
    open_details_on_activate, CardBadge, CardContent, CatalogCard,
    DetailContent, IdeaCard, ProtocolBrowser, EdgePicture, DesktopScene, LogoTile, ProtocolSummary, RiceCard,
    ScreenshotGallery,
};

const TILE_SIZE: i32 = 96;
/// The desktop whose page shows popular Shell extensions.
const GNOME_DESKTOP: &str = "gnome";
const EXTENSION_COUNT: usize = 6;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Lookbook/ui/pages/item_detail_page.ui")]
    pub struct ItemDetailPage {
        #[template_child]
        pub tile: gtk::TemplateChild<LogoTile>,
        #[template_child]
        pub name_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub run_button: gtk::TemplateChild<gtk::Button>,
        #[template_child]
        pub rices_section: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub rices_title_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub rices_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub banner_art: gtk::TemplateChild<gtk::Stack>,
        #[template_child]
        pub banner_picture: gtk::TemplateChild<EdgePicture>,
        #[template_child]
        pub badge_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub tagline_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub chips_box: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub scene: gtk::TemplateChild<DesktopScene>,
        #[template_child]
        pub gallery_section: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub gallery: gtk::TemplateChild<ScreenshotGallery>,
        #[template_child]
        pub facts_title_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub facts_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub extensions_section: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub extensions_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub ideas_section: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub ideas_title_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub ideas_note_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub ideas_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub versus_section: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub versus_title_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub versus_grid: gtk::TemplateChild<gtk::Grid>,
        #[template_child]
        pub similar_section: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub similar_title_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub similar_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub protocols_section: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub protocols_summary_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub checks_box: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub protocol_browser: gtk::TemplateChild<ProtocolBrowser>,
        pub website: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ItemDetailPage {
        const NAME: &'static str = "ItemDetailPage";
        type Type = super::ItemDetailPage;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            LogoTile::ensure_type();
            DesktopScene::ensure_type();
            EdgePicture::ensure_type();
            ScreenshotGallery::ensure_type();
            ProtocolBrowser::ensure_type();
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ItemDetailPage {
        fn constructed(&self) {
            self.parent_constructed();
            open_details_on_activate(&self.similar_flowbox);
        }
    }

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
    pub fn new(content: &DetailContent, catalog: &Catalog, system: &SystemInfo) -> Self {
        let page: Self = glib::Object::builder()
            .property("title", &content.title)
            .build();

        page.show_header(content);
        page.show_screenshots(content, catalog);
        page.load_banner_screenshot(catalog.hosted_screenshot(content.item.kind, &content.item.id));
        page.show_facts(content);
        page.show_rices(content, catalog);
        page.show_extensions(content);
        page.show_ideas(content, catalog);
        page.show_comparison(content, catalog, system);
        page.show_similar(content, catalog, system);
        page.show_protocols(content, catalog);
        page
    }

    fn show_header(&self, content: &DetailContent) {
        let imp = self.imp();
        imp.tile.set_tile_size(TILE_SIZE);
        imp.tile.set_content(&content.tile);
        imp.name_label.set_label(&content.title);
        imp.tagline_label.set_label(&content.tagline);
        imp.scene.set_scene_color(content.scene, &content.color);
        imp.website.replace(content.website.clone());
        let target = content.item.to_action_target().to_variant();
        imp.run_button.set_action_target_value(Some(&target));

        for chip in &content.chips {
            imp.chips_box.append(&chip_label(chip));
        }
        self.show_badge(content.badge);
    }

    fn show_badge(&self, badge: Option<CardBadge>) {
        CardBadge::show_on(badge, &self.imp().badge_label);
    }

    /// The Screenshots gallery. Hidden when the item has no screenshot.
    fn show_screenshots(&self, content: &DetailContent, catalog: &Catalog) {
        let imp = self.imp();
        let screenshots = catalog.gallery(content.item.kind, &content.item.id);
        imp.gallery.set_screenshots(&screenshots);
        imp.gallery_section.set_visible(!screenshots.is_empty());
    }

    /// Swaps the banner's drawn scene for the item's real screenshot, in the
    /// same place, once it loads.
    fn load_banner_screenshot(&self, hosted: Option<&HostedScreenshot>) {
        let Some(hosted) = hosted.cloned() else {
            return;
        };
        load_texture_for(self, &hosted.url.clone(), 0, "the banner screenshot", move |page, texture| {
            let imp = page.imp();
            let name = imp.name_label.label();
            imp.banner_picture.show_screenshot(&texture, &name, &hosted);
            imp.banner_art.set_visible_child_name("screenshot");
        });
    }

    /// Compositors only: the rices that run on it. Hidden when there are none.
    fn show_rices(&self, content: &DetailContent, catalog: &Catalog) {
        if content.item.kind != ItemKind::Compositor {
            return;
        }
        let imp = self.imp();
        let rices: Vec<_> = catalog
            .rices
            .iter()
            .filter(|rice| rice.runs_on(&content.item.id))
            .collect();
        if rices.is_empty() {
            return;
        }

        imp.rices_title_label
            .set_label(&gettext("Rices for {}").replace("{}", &content.title));
        for rice in rices {
            append_to_grid(&imp.rices_flowbox, &RiceCard::new(rice, catalog));
        }
        imp.rices_section.set_visible(true);
    }

    fn show_facts(&self, content: &DetailContent) {
        let imp = self.imp();
        imp.facts_title_label.set_label(&content.facts_title);
        for fact in &content.facts {
            append_to_grid(&imp.facts_flowbox, &fact_tile(fact));
        }
    }

    /// Reads GNOME's most downloaded extensions live; the section appears when they arrive.
    fn show_extensions(&self, content: &DetailContent) {
        let is_gnome = content.item.kind == ItemKind::Desktop && content.item.id == GNOME_DESKTOP;
        if !is_gnome {
            return;
        }

        glib::spawn_future_local(clone!(
            #[weak(rename_to = page)]
            self,
            async move {
                let session = shared_session();
                match fetch_popular(&session, EXTENSION_COUNT).await {
                    Ok(extensions) => {
                        let imp = page.imp();
                        for extension in &extensions {
                            append_to_grid(&imp.extensions_flowbox, &extension_card(extension, &session));
                        }
                        imp.extensions_section.set_visible(!extensions.is_empty());
                    }
                    Err(error) => tracing::warn!("Could not read extensions.gnome.org: {error:#}"),
                }
            }
        ));
    }

    /// The ideas that are ready on this desktop or compositor. Distros have none.
    fn show_ideas(&self, content: &DetailContent, catalog: &Catalog) {
        let Some(setup) = setup_for(&content.item, catalog) else {
            return;
        };
        let imp = self.imp();
        let mut ready_count = 0;

        for idea in &catalog.ideas {
            let status = evaluate(idea, &setup, catalog);
            if status.is_available() {
                ready_count += 1;
                append_to_grid(&imp.ideas_flowbox, &IdeaCard::new(idea, &status));
            }
        }

        imp.ideas_title_label
            .set_label(&gettext("What You Can Build on {}").replace("{}", &content.title));
        imp.ideas_note_label.set_label(
            &gettext("{ready} of {total} ideas are built in or ready to build.")
                .replace("{ready}", &ready_count.to_string())
                .replace("{total}", &catalog.ideas.len().to_string()),
        );
        imp.ideas_section.set_visible(ready_count > 0);
    }

    fn show_comparison(&self, content: &DetailContent, catalog: &Catalog, system: &SystemInfo) {
        let Some(comparison) = compare(&content.item, catalog, system) else {
            return;
        };
        let imp = self.imp();

        imp.versus_title_label.set_label(&comparison_title(&comparison));
        self.fill_comparison_grid(&comparison);
        imp.versus_section.set_visible(true);
    }

    /// A header row with both names, then one row per compared fact.
    fn fill_comparison_grid(&self, comparison: &Comparison) {
        let grid = &self.imp().versus_grid;
        grid.attach(&versus_label("", &["caption"]), 0, 0, 1, 1);
        grid.attach(&versus_label(&comparison.yours_name, &["heading"]), 1, 0, 1, 1);
        grid.attach(&versus_label(&comparison.theirs_name, &["heading", "versus-theirs"]), 2, 0, 1, 1);

        for (index, row) in comparison.rows.iter().enumerate() {
            let line = index as i32 + 1;
            grid.attach(&versus_label(&row.label, &["versus-label"]), 0, line, 1, 1);
            grid.attach(&versus_label(&row.yours, &["versus-value"]), 1, line, 1, 1);
            grid.attach(&versus_label(&row.theirs, &["versus-value", "versus-theirs"]), 2, line, 1, 1);
        }
    }

    fn show_similar(&self, content: &DetailContent, catalog: &Catalog, system: &SystemInfo) {
        let imp = self.imp();
        let similar = similar_items(&content.item, catalog);

        for item in &similar {
            if let Some(card_content) = card_content_for(item, catalog, system) {
                imp.similar_flowbox.append(&CatalogCard::new(&card_content));
            }
        }

        imp.similar_title_label.set_label(&similar_title(content.item.kind));
        imp.similar_section.set_visible(!similar.is_empty());
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

        imp.protocol_browser.show(&overview, None);
        imp.protocols_section.set_visible(true);
    }

    fn open_website(&self) {
        let website = self.imp().website.borrow().clone();
        open_link(self, &website);
    }
}

fn card_content_for(item: &ItemRef, catalog: &Catalog, system: &SystemInfo) -> Option<CardContent> {
    match item.kind {
        ItemKind::Distro => catalog
            .find_distro(&item.id)
            .map(|distro| CardContent::for_distro(distro, system)),
        ItemKind::Desktop => catalog
            .find_desktop(&item.id)
            .map(|desktop| CardContent::for_desktop(desktop, system)),
        ItemKind::Compositor => catalog
            .find_compositor(&item.id)
            .map(|compositor| CardContent::for_compositor(compositor, system)),
    }
}

fn versus_label(text: &str, css_classes: &[&str]) -> gtk::Label {
    gtk::Label::builder()
        .label(text)
        .xalign(0.0)
        .wrap(true)
        .css_classes(css_classes.to_vec())
        .build()
}

/// For example "CachyOS vs Fedora Workstation", or a note that it is what the user runs.
fn comparison_title(comparison: &Comparison) -> String {
    if comparison.is_yours {
        return gettext("This is what you run");
    }
    gettext("{yours} vs {theirs}")
        .replace("{yours}", &comparison.yours_name)
        .replace("{theirs}", &comparison.theirs_name)
}

fn similar_title(kind: ItemKind) -> String {
    match kind {
        ItemKind::Distro => gettext("Similar Distros"),
        ItemKind::Desktop => gettext("Similar Desktops"),
        ItemKind::Compositor => gettext("Similar Compositors"),
    }
}

/// For example "63 protocols in version 0.52.1, checked on 2025-11-20. Data from wayland-explorer."
fn protocols_summary_text(protocols: &ProtocolSummary) -> String {
    gettext("{count} protocols in version {version}, checked on {date}. Data from wayland-explorer.")
        .replace("{count}", &protocols.globals.len().to_string())
        .replace("{version}", &protocols.version)
        .replace("{date}", &protocols.checked_on)
}
