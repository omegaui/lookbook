//! The Screenshots section of a details page: large screenshots one at a
//! time, with arrows, dots, a caption and the credit. Each screenshot loads
//! when it is about to be seen.

use std::cell::RefCell;
use std::collections::HashSet;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::glib;
use gtk::glib::clone;

use super::{screenshot_credit, ScreenshotPicture};
use crate::catalog::HostedScreenshot;
use crate::http::shared_session;
use crate::wallpapers::fetch_texture;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Syspedia/ui/widgets/screenshot_gallery.ui")]
    pub struct ScreenshotGallery {
        #[template_child]
        pub carousel: gtk::TemplateChild<adw::Carousel>,
        #[template_child]
        pub previous_button: gtk::TemplateChild<gtk::Button>,
        #[template_child]
        pub next_button: gtk::TemplateChild<gtk::Button>,
        #[template_child]
        pub caption_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub credit_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub dots: gtk::TemplateChild<adw::CarouselIndicatorDots>,
        pub screenshots: RefCell<Vec<HostedScreenshot>>,
        /// Pages whose screenshot has started loading.
        pub requested: RefCell<HashSet<u32>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScreenshotGallery {
        const NAME: &'static str = "ScreenshotGallery";
        type Type = super::ScreenshotGallery;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ScreenshotGallery {}

    impl WidgetImpl for ScreenshotGallery {
        fn map(&self) {
            self.parent_map();
            self.obj().load_near(self.obj().current_page());
        }
    }
    impl BoxImpl for ScreenshotGallery {}

    #[gtk::template_callbacks]
    impl ScreenshotGallery {
        #[template_callback]
        fn on_page_changed(&self, index: u32) {
            let gallery = self.obj();
            gallery.show_details_of(index);
            gallery.load_near(index);
        }

        #[template_callback]
        fn on_previous(&self, _button: &gtk::Button) {
            self.obj().step(-1);
        }

        #[template_callback]
        fn on_next(&self, _button: &gtk::Button) {
            self.obj().step(1);
        }
    }
}

glib::wrapper! {
    pub struct ScreenshotGallery(ObjectSubclass<imp::ScreenshotGallery>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl ScreenshotGallery {
    pub fn set_screenshots(&self, screenshots: &[HostedScreenshot]) {
        self.remove_pages();
        for screenshot in screenshots {
            self.imp().carousel.append(&screenshot_page(screenshot));
        }
        self.imp().screenshots.replace(screenshots.to_vec());

        self.show_arrows_and_dots(screenshots.len() > 1);
        self.show_details_of(0);
        if self.is_mapped() {
            self.load_near(0);
        }
    }

    fn remove_pages(&self) {
        let carousel = &self.imp().carousel;
        while carousel.n_pages() > 0 {
            carousel.remove(&carousel.nth_page(0));
        }
        self.imp().requested.borrow_mut().clear();
    }

    fn show_arrows_and_dots(&self, visible: bool) {
        let imp = self.imp();
        imp.previous_button.set_visible(visible);
        imp.next_button.set_visible(visible);
        imp.dots.set_visible(visible);
    }

    fn current_page(&self) -> u32 {
        self.imp().carousel.position().round() as u32
    }

    fn step(&self, offset: i32) {
        let carousel = &self.imp().carousel;
        let pages = carousel.n_pages() as i32;
        if pages == 0 {
            return;
        }
        // Wrap around, so the arrows never dead-end.
        let index = (self.current_page() as i32 + offset).rem_euclid(pages);
        carousel.scroll_to(&carousel.nth_page(index as u32), true);
    }

    /// Shows the caption and credit of the screenshot at `index`.
    fn show_details_of(&self, index: u32) {
        let imp = self.imp();
        let screenshots = imp.screenshots.borrow();
        let Some(screenshot) = screenshots.get(index as usize) else {
            return;
        };
        let caption = screenshot.caption.clone().unwrap_or_default();
        imp.caption_label.set_visible(!caption.is_empty());
        imp.caption_label.set_label(&caption);
        imp.credit_label.set_markup(&credit_markup(screenshot));
    }

    /// Loads the screenshot at `index` and the next one, so swiping on
    /// rarely shows a placeholder.
    fn load_near(&self, index: u32) {
        for page in [index, index + 1] {
            self.load_page(page);
        }
    }

    fn load_page(&self, index: u32) {
        let imp = self.imp();
        if index >= imp.carousel.n_pages() || !imp.requested.borrow_mut().insert(index) {
            return;
        }
        let Some(url) = imp.screenshots.borrow().get(index as usize).map(|shot| shot.url.clone()) else {
            return;
        };
        let Ok(picture) = imp.carousel.nth_page(index).downcast::<ScreenshotPicture>() else {
            return;
        };
        glib::spawn_future_local(clone!(
            #[weak]
            picture,
            async move {
                match fetch_texture(&shared_session(), &url).await {
                    Ok(texture) => picture.set_texture(&texture),
                    Err(error) => tracing::warn!("Could not load a gallery screenshot: {error:#}"),
                }
            }
        ));
    }
}

fn screenshot_page(screenshot: &HostedScreenshot) -> ScreenshotPicture {
    let picture = ScreenshotPicture::default();
    picture.set_show_whole(true);
    picture.set_hexpand(true);
    picture.set_tooltip_text(Some(&screenshot_credit(screenshot)));
    let label = screenshot.caption.clone().unwrap_or_else(|| gettext("Screenshot"));
    picture.update_property(&[gtk::accessible::Property::Label(&label)]);
    picture
}

/// For example "Screenshot by The Fedora Project (CC BY-SA 4.0) · Source",
/// where Source links to the page that describes the image.
fn credit_markup(screenshot: &HostedScreenshot) -> String {
    let credit = glib::markup_escape_text(&screenshot_credit(screenshot));
    let page = glib::markup_escape_text(&screenshot.page);
    let source = glib::markup_escape_text(&gettext("Source"));
    format!("{credit} · <a href=\"{page}\">{source}</a>")
}
