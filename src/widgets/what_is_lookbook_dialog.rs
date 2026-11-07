//! "What Is Lookbook?": what the app is for, what each page holds (each row
//! opens its page), and what to know about it. From the main menu.

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

use crate::config::APP_ID;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Lookbook/ui/widgets/what_is_lookbook_dialog.ui")]
    pub struct WhatIsLookbookDialog {
        #[template_child]
        pub app_icon: gtk::TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WhatIsLookbookDialog {
        const NAME: &'static str = "WhatIsLookbookDialog";
        type Type = super::WhatIsLookbookDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for WhatIsLookbookDialog {
        fn constructed(&self) {
            self.parent_constructed();
            self.app_icon.set_icon_name(Some(APP_ID));
        }
    }

    impl WidgetImpl for WhatIsLookbookDialog {}
    impl AdwDialogImpl for WhatIsLookbookDialog {}

    #[gtk::template_callbacks]
    impl WhatIsLookbookDialog {
        /// A page row has opened its page; close, so the page shows.
        #[template_callback]
        fn on_page_activated(&self, _row: &gtk::ListBoxRow) {
            self.obj().close();
        }

        #[template_callback]
        fn on_close_clicked(&self, _button: &gtk::Button) {
            self.obj().close();
        }
    }
}

glib::wrapper! {
    pub struct WhatIsLookbookDialog(ObjectSubclass<imp::WhatIsLookbookDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for WhatIsLookbookDialog {
    fn default() -> Self {
        glib::Object::new()
    }
}
