use std::cell::RefCell;

use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

use super::ScreenshotMock;
use crate::catalog::Screenshot;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/LinuxAndYou/ui/widgets/screenshot_card.ui")]
    pub struct ScreenshotCard {
        #[template_child]
        pub mock: gtk::TemplateChild<ScreenshotMock>,
        #[template_child]
        pub title_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub author_label: gtk::TemplateChild<gtk::Label>,
        pub link: RefCell<String>,
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
    impl WidgetImpl for ScreenshotCard {}
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
