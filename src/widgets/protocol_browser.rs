//! Wayland protocols to browse: a search, one chip per category, the cards
//! that match, and the protocols with no description folded away.

use std::cell::{Cell, RefCell};

use gettextrs::{gettext, ngettext};
use gtk::glib;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::append_to_grid;
use super::protocol_card::ProtocolCard;
use crate::catalog::{ProtocolCategory, ProtocolInfo, ProtocolOverview};

/// What a card is matched against.
#[derive(Debug, Clone)]
struct Entry {
    category: ProtocolCategory,
    /// Name, interface, description and uses, in lower case.
    text: String,
}

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Lookbook/ui/widgets/protocol_browser.ui")]
    pub struct ProtocolBrowser {
        #[template_child]
        pub search_entry: gtk::TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub category_box: gtk::TemplateChild<adw::WrapBox>,
        #[template_child]
        pub cards_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub empty_page: gtk::TemplateChild<adw::StatusPage>,
        #[template_child]
        pub others_expander: gtk::TemplateChild<gtk::Expander>,
        #[template_child]
        pub others_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        pub(super) entries: RefCell<Vec<Entry>>,
        pub(super) others: RefCell<Vec<String>>,
        /// None shows every category.
        pub(super) category: Cell<Option<ProtocolCategory>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProtocolBrowser {
        const NAME: &'static str = "ProtocolBrowser";
        type Type = super::ProtocolBrowser;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProtocolBrowser {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_filters();
        }
    }

    impl WidgetImpl for ProtocolBrowser {}
    impl BoxImpl for ProtocolBrowser {}

    #[gtk::template_callbacks]
    impl ProtocolBrowser {
        #[template_callback]
        fn on_search_changed(&self, _entry: &gtk::SearchEntry) {
            self.obj().refilter();
        }
    }
}

glib::wrapper! {
    pub struct ProtocolBrowser(ObjectSubclass<imp::ProtocolBrowser>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl ProtocolBrowser {
    /// `visible` is what the sandbox can see; pass None to skip the
    /// "Hidden in Flatpak" marks.
    pub fn show(&self, overview: &ProtocolOverview<'_>, visible: Option<&[String]>) {
        self.clear();
        self.show_cards(overview, visible);
        self.show_others(&overview.others);
        self.show_category_chips(&overview.described);
        self.refilter();
    }

    fn clear(&self) {
        let imp = self.imp();
        imp.cards_flowbox.remove_all();
        imp.others_flowbox.remove_all();
        while let Some(chip) = imp.category_box.first_child() {
            imp.category_box.remove(&chip);
        }
        imp.category.set(None);
        imp.search_entry.set_text("");
    }

    fn show_cards(&self, overview: &ProtocolOverview<'_>, visible: Option<&[String]>) {
        let imp = self.imp();
        let mut entries = Vec::new();
        for info in &overview.described {
            let hidden = visible.is_some_and(|visible| !visible.contains(&info.interface));
            append_to_grid(&imp.cards_flowbox, &ProtocolCard::new(info, hidden));
            entries.push(entry_for(info));
        }
        imp.entries.replace(entries);
    }

    fn show_others(&self, others: &[String]) {
        let imp = self.imp();
        for global in others {
            let chip = gtk::Label::new(Some(global));
            chip.add_css_class("protocol-chip");
            append_to_grid(&imp.others_flowbox, &chip);
        }
        imp.others.replace(others.to_vec());
    }

    /// "All" first, then each category that has protocols, in the catalog's order.
    fn show_category_chips(&self, described: &[&ProtocolInfo]) {
        let all = self.category_chip(None, &gettext("All"), "view-grid-symbolic", described.len());
        all.set_active(true);
        self.imp().category_box.append(&all);

        for category in ProtocolCategory::ALL {
            let count = described.iter().filter(|info| info.category == category).count();
            if count == 0 {
                continue;
            }
            let chip = self.category_chip(Some(category), &category.label(), category.icon_name(), count);
            chip.set_group(Some(&all));
            self.imp().category_box.append(&chip);
        }
    }

    fn category_chip(
        &self,
        category: Option<ProtocolCategory>,
        label: &str,
        icon_name: &str,
        count: usize,
    ) -> gtk::ToggleButton {
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        content.append(&gtk::Image::from_icon_name(icon_name));
        content.append(&gtk::Label::new(Some(label)));
        let count_label = gtk::Label::new(Some(&count.to_string()));
        count_label.add_css_class("dim-label");
        content.append(&count_label);

        let chip = gtk::ToggleButton::builder()
            .child(&content)
            .css_classes(["category-chip"])
            .build();
        chip.connect_toggled(clone!(
            #[weak(rename_to = browser)]
            self,
            move |chip| {
                if chip.is_active() {
                    browser.imp().category.set(category);
                    browser.refilter();
                }
            }
        ));
        chip
    }

    fn setup_filters(&self) {
        let imp = self.imp();
        imp.cards_flowbox.set_filter_func(clone!(
            #[weak(rename_to = browser)]
            self,
            #[upgrade_or]
            true,
            move |child| browser.card_matches(child.index())
        ));
        imp.others_flowbox.set_filter_func(clone!(
            #[weak(rename_to = browser)]
            self,
            #[upgrade_or]
            true,
            move |child| browser.other_matches(child.index())
        ));
    }

    fn refilter(&self) {
        let imp = self.imp();
        imp.cards_flowbox.invalidate_filter();
        imp.others_flowbox.invalidate_filter();
        self.show_empty_state();
        self.update_others_label();
    }

    fn query(&self) -> String {
        self.imp().search_entry.text().trim().to_lowercase()
    }

    fn card_matches(&self, index: i32) -> bool {
        let entries = self.imp().entries.borrow();
        let Some(entry) = entries.get(index as usize) else {
            return true;
        };
        let in_category = self.imp().category.get().is_none_or(|category| category == entry.category);
        in_category && entry.text.contains(&self.query())
    }

    /// Undescribed protocols have no category, so a category hides them all.
    fn other_matches(&self, index: i32) -> bool {
        if self.imp().category.get().is_some() {
            return false;
        }
        let others = self.imp().others.borrow();
        others
            .get(index as usize)
            .is_none_or(|global| global.to_lowercase().contains(&self.query()))
    }

    fn matching_cards(&self) -> usize {
        let count = self.imp().entries.borrow().len();
        (0..count).filter(|index| self.card_matches(*index as i32)).count()
    }

    fn matching_others(&self) -> usize {
        let count = self.imp().others.borrow().len();
        (0..count).filter(|index| self.other_matches(*index as i32)).count()
    }

    fn show_empty_state(&self) {
        let nothing = self.matching_cards() == 0 && self.matching_others() == 0;
        self.imp().empty_page.set_visible(nothing);
    }

    fn update_others_label(&self) {
        let imp = self.imp();
        let count = self.matching_others();
        imp.others_expander.set_visible(count > 0);
        imp.others_expander.set_label(Some(
            &ngettext(
                "{} more protocol, without a description yet",
                "{} more protocols, without a description yet",
                count as u32,
            )
            .replace("{}", &count.to_string()),
        ));
        // A search through the undescribed ones opens them, so matches show.
        if !self.query().is_empty() && count > 0 {
            imp.others_expander.set_expanded(true);
        }
    }
}

fn entry_for(info: &ProtocolInfo) -> Entry {
    let text = [&info.name, &info.interface, &info.description, &info.useful_for]
        .map(|part| part.to_lowercase())
        .join(" ");
    Entry {
        category: info.category,
        text,
    }
}
