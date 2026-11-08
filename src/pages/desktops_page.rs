//! The desktop environment gallery. GNOME is featured, because Lookbook is a GNOME app first.

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

use std::cell::RefCell;

use gtk::glib::clone;

use crate::catalog::{Catalog, Desktop, HostedScreenshot};
use crate::http::shared_session;
use crate::fits_together::{page_intro, IntroKind};
use crate::system::SystemInfo;
use crate::wallpapers::fetch_texture;
use crate::widgets::{
    chip_label, open_details_on_activate, scene_for_desktop, screenshot_credit, EdgePicture, IntroBanner,
    DesktopScene, LogoTile, ShowcaseCard, ShowcaseContent, TileContent,
};

const FEATURED_DESKTOP: &str = "gnome";
const FEATURED_TILE_SIZE: i32 = 72;
/// The toolkit filter's toggle that shows every desktop.
const ALL_TOOLKITS: &str = "all";

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Lookbook/ui/pages/desktops_page.ui")]
    pub struct DesktopsPage {
        #[template_child]
        pub intro: gtk::TemplateChild<IntroBanner>,
        #[template_child]
        pub featured_banner: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub featured_tile: gtk::TemplateChild<LogoTile>,
        #[template_child]
        pub featured_name_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub featured_chips_box: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub featured_art: gtk::TemplateChild<gtk::Stack>,
        #[template_child]
        pub featured_scene: gtk::TemplateChild<DesktopScene>,
        #[template_child]
        pub featured_picture: gtk::TemplateChild<EdgePicture>,
        #[template_child]
        pub toolkit_filter: gtk::TemplateChild<adw::ToggleGroup>,
        #[template_child]
        pub desktops_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        /// GNOME's screenshot, loaded the first time the banner is on screen.
        pub featured_screenshot: RefCell<Option<HostedScreenshot>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DesktopsPage {
        const NAME: &'static str = "DesktopsPage";
        type Type = super::DesktopsPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            IntroBanner::ensure_type();
            LogoTile::ensure_type();
            DesktopScene::ensure_type();
            EdgePicture::ensure_type();
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DesktopsPage {
        fn constructed(&self) {
            self.parent_constructed();
            open_details_on_activate(&self.desktops_flowbox);
            self.obj().filter_by_toolkit();
            self.obj().load_featured_screenshot_when_shown();
        }
    }

    #[gtk::template_callbacks]
    impl DesktopsPage {
        #[template_callback]
        fn on_toolkit_changed(&self) {
            self.desktops_flowbox.invalidate_filter();
        }
    }

    impl WidgetImpl for DesktopsPage {}
    impl BinImpl for DesktopsPage {}
}

glib::wrapper! {
    pub struct DesktopsPage(ObjectSubclass<imp::DesktopsPage>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl DesktopsPage {
    pub fn show_catalog(&self, catalog: &Catalog, system: &SystemInfo) {
        self.imp().intro.show(&page_intro(IntroKind::Desktops, catalog));
        self.show_featured_desktop(catalog);
        self.show_desktops(catalog, system);
    }

    /// Hides the banner if the catalog ever stops listing GNOME.
    fn show_featured_desktop(&self, catalog: &Catalog) {
        let imp = self.imp();
        let Some(desktop) = catalog.find_desktop(FEATURED_DESKTOP) else {
            imp.featured_banner.set_visible(false);
            return;
        };

        imp.featured_tile.set_tile_size(FEATURED_TILE_SIZE);
        imp.featured_tile.set_content(&TileContent {
            short: desktop.short.clone(),
            color: desktop.color.clone(),
            logo: desktop.logo.clone(),
            icon_name: None,
        });
        imp.featured_name_label.set_label(&desktop.name);
        imp.featured_scene
            .set_scene_color(scene_for_desktop(desktop.layout), &desktop.color);
        self.show_featured_chips(desktop);
        imp.featured_screenshot.replace(desktop.screenshot.clone());
    }

    fn load_featured_screenshot_when_shown(&self) {
        self.imp().featured_art.connect_map(clone!(
            #[weak(rename_to = page)]
            self,
            move |_| page.load_featured_screenshot()
        ));
    }

    /// Takes the screenshot out of the page, so it loads only once.
    fn load_featured_screenshot(&self) {
        let Some(screenshot) = self.imp().featured_screenshot.take() else {
            return;
        };
        glib::spawn_future_local(clone!(
            #[weak(rename_to = page)]
            self,
            async move {
                match fetch_texture(&shared_session(), &screenshot.url).await {
                    Ok(texture) => page.show_featured_screenshot(&texture, &screenshot),
                    Err(error) => tracing::warn!("Could not load GNOME's screenshot: {error:#}"),
                }
            }
        ));
    }

    fn show_featured_screenshot(&self, texture: &gtk::gdk::Texture, screenshot: &HostedScreenshot) {
        let imp = self.imp();
        imp.featured_picture.set_texture(Some(texture));
        imp.featured_picture
            .set_tooltip_text(Some(&screenshot_credit(screenshot)));
        imp.featured_art.set_visible_child_name("screenshot");
    }

    fn show_featured_chips(&self, desktop: &Desktop) {
        let chips_box = &self.imp().featured_chips_box;
        for chip in [&desktop.compositor, &desktop.toolkit, &desktop.wayland] {
            chips_box.append(&chip_label(chip));
        }
    }

    fn show_desktops(&self, catalog: &Catalog, system: &SystemInfo) {
        for desktop in &catalog.desktops {
            self.imp()
                .desktops_flowbox
                .append(&ShowcaseCard::new(&ShowcaseContent::for_desktop(desktop, system)));
        }
    }

    /// Shows only the cards whose toolkit matches the chosen toggle.
    fn filter_by_toolkit(&self) {
        let imp = self.imp();
        let filter = imp.toolkit_filter.get();
        imp.desktops_flowbox.set_filter_func(move |child| {
            let chosen = filter.active_name();
            let chosen = chosen.as_deref().unwrap_or(ALL_TOOLKITS);
            if chosen == ALL_TOOLKITS {
                return true;
            }
            child
                .child()
                .and_downcast::<ShowcaseCard>()
                .and_then(|card| card.toolkit_id())
                .is_some_and(|toolkit| toolkit == chosen)
        });
    }
}
