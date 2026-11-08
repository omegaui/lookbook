//! A large preview of one wallpaper, with Set as Wallpaper, Save and Source.

use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::glib::clone;
use gtk::{gdk, gio, glib};

use crate::wallpapers::{fetch_bytes, file_name_from_url, set_wallpaper, Wallpaper, WallpaperOutcome};

/// What the dialog needs besides the wallpaper itself.
pub struct WallpaperDialogContext {
    pub session: soup::Session,
    /// Who made the collection, for example "Collected by orangc."
    pub credit: String,
    /// The collection's page, opened by the Source button.
    pub source_link: String,
    /// Shown when the user's portal may not be able to set wallpapers.
    pub hint: Option<String>,
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Syspedia/ui/widgets/wallpaper_dialog.ui")]
    pub struct WallpaperDialog {
        #[template_child]
        pub toast_overlay: gtk::TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub picture: gtk::TemplateChild<gtk::Picture>,
        #[template_child]
        pub loading_indicator: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub title_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub credit_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub save_button: gtk::TemplateChild<gtk::Button>,
        #[template_child]
        pub set_button: gtk::TemplateChild<gtk::Button>,
        #[template_child]
        pub hint_label: gtk::TemplateChild<gtk::Label>,
        pub wallpaper: RefCell<Option<Wallpaper>>,
        pub session: RefCell<Option<soup::Session>>,
        pub source_link: RefCell<String>,
        /// The full image, once downloaded.
        pub image: RefCell<Option<glib::Bytes>>,
    }

    impl std::fmt::Debug for WallpaperDialog {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.debug_struct("WallpaperDialog").finish_non_exhaustive()
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WallpaperDialog {
        const NAME: &'static str = "WallpaperDialog";
        type Type = super::WallpaperDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for WallpaperDialog {}
    impl WidgetImpl for WallpaperDialog {}
    impl AdwDialogImpl for WallpaperDialog {}

    #[gtk::template_callbacks]
    impl WallpaperDialog {
        #[template_callback]
        fn on_set_clicked(&self, _button: &gtk::Button) {
            self.obj().apply_wallpaper();
        }

        #[template_callback]
        fn on_save_clicked(&self, _button: &gtk::Button) {
            self.obj().save_wallpaper();
        }

        #[template_callback]
        fn on_source_clicked(&self, _button: &gtk::Button) {
            self.obj().open_source();
        }
    }
}

glib::wrapper! {
    pub struct WallpaperDialog(ObjectSubclass<imp::WallpaperDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

impl WallpaperDialog {
    pub fn new(
        wallpaper: &Wallpaper,
        thumbnail: Option<&gdk::Paintable>,
        context: WallpaperDialogContext,
    ) -> Self {
        let dialog: Self = glib::Object::builder()
            .property("title", &wallpaper.title)
            .build();

        dialog.show_details(wallpaper, thumbnail, &context);
        dialog.store_context(wallpaper, context);
        dialog.load_full_image();
        dialog
    }

    fn show_details(
        &self,
        wallpaper: &Wallpaper,
        thumbnail: Option<&gdk::Paintable>,
        context: &WallpaperDialogContext,
    ) {
        let imp = self.imp();
        imp.picture.set_paintable(thumbnail);
        imp.picture.set_alternative_text(Some(&wallpaper.title));
        imp.title_label.set_label(&wallpaper.title);
        imp.credit_label.set_label(&context.credit);
        imp.hint_label.set_label(context.hint.as_deref().unwrap_or_default());
        imp.hint_label.set_visible(context.hint.is_some());
    }

    fn store_context(&self, wallpaper: &Wallpaper, context: WallpaperDialogContext) {
        let imp = self.imp();
        imp.wallpaper.replace(Some(wallpaper.clone()));
        imp.session.replace(Some(context.session));
        imp.source_link.replace(context.source_link);
    }

    /// Shows the thumbnail at once, then swaps in the full image when it arrives.
    fn load_full_image(&self) {
        self.set_busy(true);

        glib::spawn_future_local(clone!(
            #[weak(rename_to = dialog)]
            self,
            async move {
                let result = dialog.download_image().await;
                dialog.set_busy(false);
                match result {
                    Ok(bytes) => dialog.show_full_image(&bytes),
                    Err(error) => {
                        tracing::warn!("Could not download the wallpaper: {error:#}");
                        dialog.offer_retry();
                    }
                }
            }
        ));
    }

    /// Set and Save need the full image, so they stay off until a retry brings it.
    fn offer_retry(&self) {
        let imp = self.imp();
        imp.set_button.set_sensitive(false);
        imp.save_button.set_sensitive(false);

        let toast = adw::Toast::builder()
            .title(gettext("Could not download the full image"))
            .button_label(gettext("Retry"))
            .timeout(0)
            .build();
        toast.connect_button_clicked(clone!(
            #[weak(rename_to = dialog)]
            self,
            move |_| dialog.load_full_image()
        ));
        imp.toast_overlay.add_toast(toast);
    }

    async fn download_image(&self) -> anyhow::Result<glib::Bytes> {
        let imp = self.imp();
        let session = imp.session.borrow().clone();
        let url = imp.wallpaper.borrow().as_ref().map(|wallpaper| wallpaper.full_url.clone());
        let (Some(session), Some(url)) = (session, url) else {
            anyhow::bail!("the dialog has no wallpaper");
        };

        fetch_bytes(&session, &url).await
    }

    fn show_full_image(&self, bytes: &glib::Bytes) {
        let imp = self.imp();
        match gdk::Texture::from_bytes(bytes) {
            Ok(texture) => imp.picture.set_paintable(Some(&texture)),
            Err(error) => tracing::warn!("Could not decode the wallpaper: {error}"),
        }
        imp.image.replace(Some(bytes.clone()));
    }

    /// While the full image downloads, the thumbnail is blurred under a loading
    /// pill, and the buttons wait.
    fn set_busy(&self, busy: bool) {
        let imp = self.imp();
        imp.loading_indicator.set_visible(busy);
        imp.set_button.set_sensitive(!busy);
        imp.save_button.set_sensitive(!busy);

        if busy {
            imp.picture.add_css_class("loading");
        } else {
            imp.picture.remove_css_class("loading");
        }
    }

    fn image_and_file_name(&self) -> Option<(glib::Bytes, String)> {
        let imp = self.imp();
        let image = imp.image.borrow().clone()?;
        let file_name = imp
            .wallpaper
            .borrow()
            .as_ref()
            .map(|wallpaper| file_name_from_url(&wallpaper.full_url))?;
        Some((image, file_name))
    }

    fn apply_wallpaper(&self) {
        let Some((image, file_name)) = self.image_and_file_name() else {
            return;
        };

        glib::spawn_future_local(clone!(
            #[weak(rename_to = dialog)]
            self,
            async move {
                match set_wallpaper(&image, &file_name).await {
                    Ok(WallpaperOutcome::Set) => dialog.toast(&gettext("Wallpaper set")),
                    Ok(WallpaperOutcome::Cancelled) => dialog.toast(&gettext("Wallpaper not changed")),
                    Err(error) => {
                        tracing::warn!("Could not set the wallpaper: {error:#}");
                        dialog.toast(&gettext("Your desktop could not set it. Try Save instead."));
                    }
                }
            }
        ));
    }

    fn save_wallpaper(&self) {
        let Some((image, file_name)) = self.image_and_file_name() else {
            return;
        };
        let file_dialog = gtk::FileDialog::builder()
            .title(gettext("Save Wallpaper"))
            .initial_name(file_name)
            .build();
        let window = self.root().and_downcast::<gtk::Window>();

        glib::spawn_future_local(clone!(
            #[weak(rename_to = dialog)]
            self,
            async move {
                let Ok(file) = file_dialog.save_future(window.as_ref()).await else {
                    return;
                };
                let result = file
                    .replace_contents_future(image, None, false, gio::FileCreateFlags::REPLACE_DESTINATION)
                    .await;

                match result {
                    Ok(_) => dialog.toast(&gettext("Wallpaper saved")),
                    Err((_, error)) => {
                        tracing::warn!("Could not save the wallpaper: {error}");
                        dialog.toast(&gettext("Could not save the wallpaper"));
                    }
                }
            }
        ));
    }

    fn open_source(&self) {
        let link = self.imp().source_link.borrow().clone();
        super::open_link(self, &link);
    }

    fn toast(&self, message: &str) {
        self.imp().toast_overlay.add_toast(adw::Toast::new(message));
    }
}
