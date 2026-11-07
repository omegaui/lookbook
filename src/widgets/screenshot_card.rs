use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;

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
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScreenshotCard {
        const NAME: &'static str = "ScreenshotCard";
        type Type = super::ScreenshotCard;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            ScreenshotMock::ensure_type();
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ScreenshotCard {}
    impl WidgetImpl for ScreenshotCard {}
    impl BoxImpl for ScreenshotCard {}
}

glib::wrapper! {
    /// A Wall of fame entry: the screenshot and who shared it.
    pub struct ScreenshotCard(ObjectSubclass<imp::ScreenshotCard>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
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
        imp.mock
            .update_property(&[gtk::accessible::Property::Label(&screenshot.title)]);
        imp.title_label.set_label(&screenshot.title);
        imp.author_label.set_label(&screenshot.author);
    }
}
