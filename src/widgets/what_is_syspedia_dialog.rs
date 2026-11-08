//! "What Is Syspedia?": what the app is for, what each page holds (each row
//! opens its page), and what to know about it. From the main menu.

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

use crate::config::APP_ID;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Syspedia/ui/widgets/what_is_syspedia_dialog.ui")]
    pub struct WhatIsSyspediaDialog {
        #[template_child]
        pub app_icon: gtk::TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WhatIsSyspediaDialog {
        const NAME: &'static str = "WhatIsSyspediaDialog";
        type Type = super::WhatIsSyspediaDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for WhatIsSyspediaDialog {
        fn constructed(&self) {
            self.parent_constructed();
            self.app_icon.set_icon_name(Some(APP_ID));
        }
    }

    impl WidgetImpl for WhatIsSyspediaDialog {}
    impl AdwDialogImpl for WhatIsSyspediaDialog {}

    #[gtk::template_callbacks]
    impl WhatIsSyspediaDialog {
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
    pub struct WhatIsSyspediaDialog(ObjectSubclass<imp::WhatIsSyspediaDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for WhatIsSyspediaDialog {
    fn default() -> Self {
        glib::Object::new()
    }
}
