use std::cell::RefCell;

use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

use gtk::glib::clone;

use super::{screenshot_credit, ScreenshotMock};
use crate::catalog::{HostedScreenshot, Screenshot};
use crate::http::shared_session;
use crate::wallpapers::fetch_texture;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Lookbook/ui/widgets/screenshot_card.ui")]
    pub struct ScreenshotCard {
        #[template_child]
        pub mock: gtk::TemplateChild<ScreenshotMock>,
        #[template_child]
        pub title_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub author_label: gtk::TemplateChild<gtk::Label>,
        pub link: RefCell<String>,
        /// The real screenshot to load the first time the card is on screen.
        pub hosted: RefCell<Option<HostedScreenshot>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScreenshotCard {
        const NAME: &'static str = "ScreenshotCard";
        type Type = super::ScreenshotCard;
        type ParentType = gtk::Button;

        fn class_init(klass: &mut Self::Class) {
            ScreenshotMock::ensure_type();
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ScreenshotCard {}

    impl WidgetImpl for ScreenshotCard {
        fn map(&self) {
            self.parent_map();
            self.obj().load_hosted_screenshot();
        }
    }
    impl ButtonImpl for ScreenshotCard {}

    #[gtk::template_callbacks]
    impl ScreenshotCard {
        #[template_callback]
        fn on_clicked(&self, _button: &gtk::Button) {
            self.obj().open_link();
        }
    }
}

glib::wrapper! {
    /// A Hall of fame entry. Clicking it opens the place with the best real
    /// screenshots of that look, such as Hyprland's Hall of Fame.
    pub struct ScreenshotCard(ObjectSubclass<imp::ScreenshotCard>)
        @extends gtk::Widget, gtk::Button,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl ScreenshotCard {
    pub fn new(screenshot: &Screenshot) -> Self {
        let card: Self = glib::Object::new();
        card.show_screenshot(screenshot);
        card
    }

    /// A Hall of fame entry with a real screenshot of `name`. The drawn look,
    /// if any, shows until the image arrives; the caption credits the image.
    pub fn with_hosted(screenshot: Option<&Screenshot>, name: &str, hosted: &HostedScreenshot, link: &str) -> Self {
        let card: Self = glib::Object::new();
        if let Some(screenshot) = screenshot {
            card.show_screenshot(screenshot);
        }
        let imp = card.imp();
        imp.title_label.set_label(name);
        imp.author_label
            .set_label(&gettext("by {}").replace("{}", &hosted.credit));
        imp.link.replace(link.to_owned());
        imp.hosted.replace(Some(hosted.clone()));
        card.set_tooltip_text(Some(&screenshot_credit(hosted)));
        card.update_property(&[gtk::accessible::Property::Label(name)]);
        card
    }

    /// Takes the screenshot out of the card, so it loads only once.
    fn load_hosted_screenshot(&self) {
        let Some(hosted) = self.imp().hosted.take() else {
            return;
        };
        glib::spawn_future_local(clone!(
            #[weak(rename_to = card)]
            self,
            async move {
                match fetch_texture(&shared_session(), &hosted.url).await {
                    Ok(texture) => card.imp().mock.set_texture(&texture),
                    Err(error) => tracing::warn!("Could not load a Hall of Fame screenshot: {error:#}"),
                }
            }
        ));
    }

    fn show_screenshot(&self, screenshot: &Screenshot) {
        let imp = self.imp();
        imp.mock.set_look(&screenshot.mock);
        imp.title_label.set_label(&screenshot.title);
        imp.author_label.set_label(&screenshot.author);
        imp.link.replace(screenshot.link.clone());

        self.set_tooltip_text(Some(&gettext("See more setups like this")));
        self.update_property(&[gtk::accessible::Property::Label(&screenshot.title)]);
    }

    fn open_link(&self) {
        let link = self.imp().link.borrow().clone();
        let window = self.root().and_downcast::<gtk::Window>();

        gtk::UriLauncher::new(&link).launch(window.as_ref(), gio::Cancellable::NONE, |result| {
            if let Err(error) = result {
                tracing::warn!("Could not open the Hall of fame link: {error}");
            }
        });
    }
}
