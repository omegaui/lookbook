use std::cell::RefCell;

use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

use gtk::glib::clone;

use super::ScreenshotPicture;
use crate::catalog::{Contest, FamedRice};
use crate::http::shared_session;
use crate::wallpapers::fetch_texture;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Lookbook/ui/widgets/screenshot_card.ui")]
    pub struct ScreenshotCard {
        #[template_child]
        pub picture: gtk::TemplateChild<ScreenshotPicture>,
        #[template_child]
        pub place_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub title_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub author_label: gtk::TemplateChild<gtk::Label>,
        pub link: RefCell<String>,
        /// The screenshot to load the first time the card is on screen.
        pub image_url: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScreenshotCard {
        const NAME: &'static str = "ScreenshotCard";
        type Type = super::ScreenshotCard;
        type ParentType = gtk::Button;

        fn class_init(klass: &mut Self::Class) {
            ScreenshotPicture::ensure_type();
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
            self.obj().load_screenshot();
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
    /// A Hall of fame winner: its screenshot, place and author. Clicking it
    /// opens the rice's dotfiles.
    pub struct ScreenshotCard(ObjectSubclass<imp::ScreenshotCard>)
        @extends gtk::Widget, gtk::Button,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl ScreenshotCard {
    pub fn new(contest: &Contest, rice: &FamedRice) -> Self {
        let card: Self = glib::Object::new();
        card.show_rice(contest, rice);
        card
    }

    fn show_rice(&self, contest: &Contest, rice: &FamedRice) {
        let imp = self.imp();
        imp.title_label.set_label(&rice.name);
        imp.author_label
            .set_label(&gettext("by {}").replace("{}", &rice.author));
        imp.place_label.set_label(&place_text(contest, rice));
        imp.link.replace(rice.link.clone());
        imp.image_url.replace(Some(rice.image.clone()));

        self.set_tooltip_text(Some(&credit_text(contest, rice)));
        self.update_property(&[gtk::accessible::Property::Label(&rice.name)]);
    }

    /// Takes the address out of the card, so the screenshot loads only once.
    fn load_screenshot(&self) {
        let Some(url) = self.imp().image_url.take() else {
            return;
        };
        glib::spawn_future_local(clone!(
            #[weak(rename_to = card)]
            self,
            async move {
                match fetch_texture(&shared_session(), &url).await {
                    Ok(texture) => card.imp().picture.set_texture(&texture),
                    Err(error) => tracing::warn!("Could not load a Hall of Fame screenshot: {error:#}"),
                }
            }
        ));
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

/// For example "#1 · Moon, June 2026".
fn place_text(contest: &Contest, rice: &FamedRice) -> String {
    gettext("{place} · {contest}, {date}")
        .replace("{place}", short_place(&rice.place))
        .replace("{contest}", &contest.name)
        .replace("{date}", &contest.date)
}

/// "#2 (ex aequo)" becomes "#2"; the pill has little room.
fn short_place(place: &str) -> &str {
    place.split_whitespace().next().unwrap_or(place)
}

fn credit_text(contest: &Contest, rice: &FamedRice) -> String {
    gettext("Screenshot by {author}, from Hyprland's {contest} ricing contest. Click to see the dotfiles.")
        .replace("{author}", &rice.author)
        .replace("{contest}", &contest.name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ties_drop_the_note() {
        assert_eq!(short_place("#2 (ex aequo)"), "#2");
        assert_eq!(short_place("#1"), "#1");
    }
}
