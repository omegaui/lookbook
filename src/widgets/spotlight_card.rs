use std::cell::RefCell;

use gtk::glib;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::{chip_label, EdgePicture, DesktopScene, LogoTile, SpotlightContent};
use crate::catalog::HostedScreenshot;
use crate::http::shared_session;
use crate::wallpapers::fetch_texture;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Lookbook/ui/widgets/spotlight_card.ui")]
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
        #[template_child]
        pub run_button: gtk::TemplateChild<gtk::Button>,
        #[template_child]
        pub art_stack: gtk::TemplateChild<gtk::Stack>,
        #[template_child]
        pub scene: gtk::TemplateChild<DesktopScene>,
        #[template_child]
        pub picture: gtk::TemplateChild<EdgePicture>,
        /// Taken out when it starts loading, so it loads only once.
        pub screenshot: RefCell<Option<HostedScreenshot>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SpotlightCard {
        const NAME: &'static str = "SpotlightCard";
        type Type = super::SpotlightCard;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            LogoTile::ensure_type();
            DesktopScene::ensure_type();
            EdgePicture::ensure_type();
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SpotlightCard {}

    impl WidgetImpl for SpotlightCard {
        /// The screenshot loads the first time the spotlight is on screen.
        fn map(&self) {
            self.parent_map();
            self.obj().load_screenshot();
        }
    }
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
        let target = content.item.to_action_target().to_variant();
        imp.details_button.set_action_target_value(Some(&target));
        imp.run_button.set_action_target_value(Some(&target));
        imp.scene.set_scene(content.scene, content.accent);
        imp.screenshot.replace(content.screenshot.clone());

        for chip in &content.chips {
            imp.chips_box.append(&chip_label(chip));
        }
    }

    fn load_screenshot(&self) {
        let Some(screenshot) = self.imp().screenshot.take() else {
            return;
        };
        glib::spawn_future_local(clone!(
            #[weak(rename_to = card)]
            self,
            async move {
                match fetch_texture(&shared_session(), &screenshot.url).await {
                    Ok(texture) => {
                        let imp = card.imp();
                        let name = imp.name_label.label();
                        imp.picture.show_screenshot(&texture, &name, &screenshot);
                        imp.art_stack.set_visible_child_name("screenshot");
                    }
                    Err(error) => tracing::warn!("Could not load a spotlight screenshot: {error:#}"),
                }
            }
        ));
    }
}
