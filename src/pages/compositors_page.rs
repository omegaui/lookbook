//! The compositor gallery, with a layout filter and key protocol checks.

use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::glib::clone;

use gettextrs::gettext;

use crate::catalog::{Catalog, CompositorKind};
use crate::system::SystemInfo;
use crate::widgets::{open_details_on_activate, CardContent, CatalogCard, RiceCard};

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
        #[template_child]
        pub rices_section: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub rice_filter: gtk::TemplateChild<adw::ToggleGroup>,
        #[template_child]
        pub rices_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        /// The compositors each rice card runs on, in the same order as the cards.
        pub rice_compositors: RefCell<Vec<Vec<String>>>,
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
            self.obj().setup_rice_filter();
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

        self.show_rices(catalog);
    }

    fn show_rices(&self, catalog: &Catalog) {
        let imp = self.imp();
        if catalog.rices.is_empty() {
            return;
        }

        for rice in &catalog.rices {
            imp.rices_flowbox.append(&RiceCard::new(rice, catalog));
        }
        let compositors = catalog.rices.iter().map(|rice| rice.compositors.clone()).collect();
        imp.rice_compositors.replace(compositors);

        self.add_rice_toggles(catalog);
        imp.rices_section.set_visible(true);
    }

    /// "All", then each compositor that at least one rice runs on, in catalog order.
    fn add_rice_toggles(&self, catalog: &Catalog) {
        let filter = &self.imp().rice_filter;
        filter.add(adw::Toggle::builder().name(SHOW_ALL).label(gettext("All")).build());

        let with_rices = catalog
            .compositors
            .iter()
            .filter(|compositor| catalog.rices.iter().any(|rice| rice.runs_on(&compositor.id)));
        for compositor in with_rices {
            let toggle = adw::Toggle::builder()
                .name(compositor.id.as_str())
                .label(compositor.name.as_str())
                .build();
            filter.add(toggle);
        }
        filter.set_active_name(Some(SHOW_ALL));
    }

    fn setup_rice_filter(&self) {
        let imp = self.imp();

        imp.rices_flowbox.set_filter_func(clone!(
            #[weak(rename_to = page)]
            self,
            #[upgrade_or]
            true,
            move |child| page.rice_matches(child.index())
        ));

        imp.rice_filter.connect_active_name_notify(clone!(
            #[weak(rename_to = page)]
            self,
            move |_| page.imp().rices_flowbox.invalidate_filter()
        ));
    }

    fn rice_matches(&self, index: i32) -> bool {
        let imp = self.imp();
        let Some(active) = imp.rice_filter.active_name() else {
            return true;
        };
        if active == SHOW_ALL {
            return true;
        }
        imp.rice_compositors
            .borrow()
            .get(index as usize)
            .is_some_and(|compositors| compositors.iter().any(|id| *id == active))
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
