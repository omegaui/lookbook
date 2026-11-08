//! The distro gallery, with search and filters.

use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::{gettext, ngettext};
use gtk::glib;
use gtk::glib::clone;

use crate::catalog::{Catalog, Distro, DistroFamily, ReleaseModel};
use crate::fits_together::{page_intro, IntroKind};
use crate::system::SystemInfo;
use crate::widgets::{add_family_toggles, open_details_on_activate, IntroBanner, ShowcaseCard, ShowcaseContent};

/// Opens a new issue on the project, titled for a distro suggestion.
const SUGGEST_DISTRO_URL: &str = "https://github.com/omegaui/lookbook/issues/new?title=Suggest%20a%20distro%3A%20";

/// The toggle name that turns a filter off.
const SHOW_ALL: &str = "all";

/// What the filters look at for one distro card.
#[derive(Debug, Clone)]
struct FilterEntry {
    family: DistroFamily,
    release: ReleaseModel,
    /// Name, tagline and package manager in lower case, for search.
    search_text: String,
}

impl FilterEntry {
    fn new(distro: &Distro) -> Self {
        let search_text = [&distro.name, &distro.tagline, &distro.package_manager]
            .map(|text| text.to_lowercase())
            .join(" ");

        Self {
            family: distro.family,
            release: distro.release,
            search_text,
        }
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Lookbook/ui/pages/distros_page.ui")]
    pub struct DistrosPage {
        #[template_child]
        pub intro: gtk::TemplateChild<IntroBanner>,
        #[template_child]
        pub count_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub suggest_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub search_entry: gtk::TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub family_toggles: gtk::TemplateChild<adw::ToggleGroup>,
        #[template_child]
        pub release_toggles: gtk::TemplateChild<adw::ToggleGroup>,
        #[template_child]
        pub distros_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub empty_page: gtk::TemplateChild<adw::StatusPage>,
        /// One entry per card in `distros_flowbox`, in the same order.
        pub(super) entries: RefCell<Vec<FilterEntry>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DistrosPage {
        const NAME: &'static str = "DistrosPage";
        type Type = super::DistrosPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            IntroBanner::ensure_type();
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DistrosPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_filters();
            open_details_on_activate(&self.distros_flowbox);
        }
    }

    impl WidgetImpl for DistrosPage {}
    impl BinImpl for DistrosPage {}
}

glib::wrapper! {
    pub struct DistrosPage(ObjectSubclass<imp::DistrosPage>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl DistrosPage {
    pub fn show_catalog(&self, catalog: &Catalog, system: &SystemInfo) {
        self.imp().intro.show(&page_intro(IntroKind::Distros, catalog));
        let imp = self.imp();

        for distro in &catalog.distros {
            let content = ShowcaseContent::for_distro(distro, catalog, system);
            imp.distros_flowbox.append(&ShowcaseCard::new(&content));
        }

        let entries = catalog.distros.iter().map(FilterEntry::new).collect();
        imp.entries.replace(entries);
        self.update_results();
        self.show_suggest_note();
    }

    fn setup_filters(&self) {
        let imp = self.imp();
        add_family_toggles(&imp.family_toggles);

        imp.distros_flowbox.set_filter_func(clone!(
            #[weak(rename_to = page)]
            self,
            #[upgrade_or]
            true,
            move |child| page.card_matches(child.index())
        ));

        imp.search_entry.connect_search_changed(clone!(
            #[weak(rename_to = page)]
            self,
            move |_| page.update_results()
        ));

        for toggles in [&imp.family_toggles, &imp.release_toggles] {
            toggles.connect_active_name_notify(clone!(
                #[weak(rename_to = page)]
                self,
                move |_| page.update_results()
            ));
        }
    }

    /// Says the list is a selection, with a link to suggest a missing distro.
    fn show_suggest_note(&self) {
        let link = glib::markup_escape_text(SUGGEST_DISTRO_URL);
        let note = gettext("There are hundreds of distros; Lookbook shows a hand-picked few. Missing one you like? <a href=\"{link}\">Suggest it</a>, and it can join the list without an app update.")
            .replace("{link}", &link);
        self.imp().suggest_label.set_markup(&note);
    }

    /// Re-applies the filters and updates the count and the empty state.
    fn update_results(&self) {
        let imp = self.imp();
        imp.distros_flowbox.invalidate_filter();

        let count = imp
            .entries
            .borrow()
            .iter()
            .filter(|entry| self.entry_matches(entry))
            .count();

        imp.count_label.set_label(&count_text(count, imp.entries.borrow().len()));
        imp.distros_flowbox.set_visible(count > 0);
        imp.empty_page.set_visible(count == 0);
    }

    fn card_matches(&self, index: i32) -> bool {
        let entries = self.imp().entries.borrow();
        match entries.get(index as usize) {
            Some(entry) => self.entry_matches(entry),
            None => true,
        }
    }

    fn entry_matches(&self, entry: &FilterEntry) -> bool {
        self.matches_family(entry) && self.matches_release(entry) && self.matches_search(entry)
    }

    fn matches_family(&self, entry: &FilterEntry) -> bool {
        let active = self.imp().family_toggles.active_name();
        match active.as_deref() {
            None | Some(SHOW_ALL) => true,
            Some(family) => entry.family.id() == family,
        }
    }

    fn matches_release(&self, entry: &FilterEntry) -> bool {
        let active = self.imp().release_toggles.active_name();
        match active.as_deref() {
            None | Some(SHOW_ALL) => true,
            Some(release) => entry.release.id() == release,
        }
    }

    fn matches_search(&self, entry: &FilterEntry) -> bool {
        let query = self.imp().search_entry.text().trim().to_lowercase();
        query.is_empty() || entry.search_text.contains(&query)
    }
}

/// "16 hand-picked distros" with no filter, "3 of 16 hand-picked distros"
/// with one, so the count never reads as every distro there is.
fn count_text(shown: usize, total: usize) -> String {
    if shown == total {
        return ngettext("{} hand-picked distro", "{} hand-picked distros", total as u32)
            .replace("{}", &total.to_string());
    }
    ngettext("{shown} of {total} hand-picked distro", "{shown} of {total} hand-picked distros", total as u32)
        .replace("{shown}", &shown.to_string())
        .replace("{total}", &total.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_count_says_the_list_is_a_selection() {
        assert_eq!(count_text(16, 16), "16 hand-picked distros");
        assert_eq!(count_text(3, 16), "3 of 16 hand-picked distros");
    }
}
