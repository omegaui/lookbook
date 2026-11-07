//! The compositor gallery, with a layout filter and key protocol checks.

use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::glib::clone;

use crate::catalog::{Catalog, CompositorKind};
use crate::system::SystemInfo;
use crate::widgets::{open_details_on_activate, CardContent, CatalogCard};

/// The toggle name that turns the filter off.
const SHOW_ALL: &str = "all";

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/LinuxAndYou/ui/pages/compositors_page.ui")]
    pub struct CompositorsPage {
        #[template_child]
        pub kind_toggles: gtk::TemplateChild<adw::ToggleGroup>,
        #[template_child]
        pub compositors_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        /// The layout of each card in `compositors_flowbox`, in the same order.
        pub kinds: RefCell<Vec<CompositorKind>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CompositorsPage {
        const NAME: &'static str = "CompositorsPage";
        type Type = super::CompositorsPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CompositorsPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_filter();
            open_details_on_activate(&self.compositors_flowbox);
        }
    }

    impl WidgetImpl for CompositorsPage {}
    impl BinImpl for CompositorsPage {}
}

glib::wrapper! {
    pub struct CompositorsPage(ObjectSubclass<imp::CompositorsPage>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl CompositorsPage {
    pub fn show_catalog(&self, catalog: &Catalog, system: &SystemInfo) {
        let imp = self.imp();

        for compositor in &catalog.compositors {
            let content = CardContent::for_compositor(compositor, system);
            imp.compositors_flowbox.append(&CatalogCard::new(&content));
        }

        let kinds = catalog.compositors.iter().map(|compositor| compositor.kind).collect();
        imp.kinds.replace(kinds);
    }

    fn setup_filter(&self) {
        let imp = self.imp();

        imp.compositors_flowbox.set_filter_func(clone!(
            #[weak(rename_to = page)]
            self,
            #[upgrade_or]
            true,
            move |child| page.card_matches(child.index())
        ));

        imp.kind_toggles.connect_active_name_notify(clone!(
            #[weak(rename_to = page)]
            self,
            move |_| page.imp().compositors_flowbox.invalidate_filter()
        ));
    }

    fn card_matches(&self, index: i32) -> bool {
        let imp = self.imp();
        let active = imp.kind_toggles.active_name();
        let kinds = imp.kinds.borrow();

        match (active.as_deref(), kinds.get(index as usize)) {
            (None | Some(SHOW_ALL), _) | (_, None) => true,
            (Some(kind), Some(card_kind)) => card_kind.id() == kind,
        }
    }
}
