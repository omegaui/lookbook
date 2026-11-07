//! Your System: what the user runs, and which ideas are ready on it.

use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::glib;
use gtk::glib::clone;

use crate::catalog::Catalog;
use crate::ideas::{evaluate, CurrentSetup};
use crate::system::SystemInfo;
use crate::widgets::{fact_tile, Fact, IdeaCard, LogoTile, TileContent};

const SUMMARY_TILE_SIZE: i32 = 64;
const SHOW_ALL: &str = "all";
const SHOW_READY: &str = "ready";

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/LinuxAndYou/ui/pages/system_page.ui")]
    pub struct SystemPage {
        #[template_child]
        pub title_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub session_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub ready_bar: gtk::TemplateChild<gtk::LevelBar>,
        #[template_child]
        pub ready_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub tiles_box: gtk::TemplateChild<gtk::Box>,
        #[template_child]
        pub facts_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub ideas_note_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub status_toggles: gtk::TemplateChild<adw::ToggleGroup>,
        #[template_child]
        pub ideas_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub visible_expander: gtk::TemplateChild<gtk::Expander>,
        #[template_child]
        pub visible_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        /// Whether each card in `ideas_flowbox` is ready, in the same order.
        pub ready_flags: RefCell<Vec<bool>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SystemPage {
        const NAME: &'static str = "SystemPage";
        type Type = super::SystemPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SystemPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_status_filter();
        }
    }

    impl WidgetImpl for SystemPage {}
    impl BinImpl for SystemPage {}
}

glib::wrapper! {
    pub struct SystemPage(ObjectSubclass<imp::SystemPage>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl SystemPage {
    pub fn show_system(&self, catalog: &Catalog, system: &SystemInfo) {
        let setup = CurrentSetup::detect(catalog, system);

        self.show_summary(catalog, system);
        self.show_logo_tiles(catalog, system, &setup);
        self.show_facts(catalog, system, &setup);
        self.show_ideas(catalog, &setup);
        self.show_visible_globals(system);
    }

    fn show_summary(&self, catalog: &Catalog, system: &SystemInfo) {
        let imp = self.imp();
        let distro = system.distro_name();
        let desktop = system.desktop_name(catalog);

        imp.title_label.set_label(&format!("{distro} + {desktop}"));
        imp.session_label.set_label(&session_text(&system.session.session_type));
    }

    /// The distro, desktop and compositor logos, for the ones the catalog knows.
    fn show_logo_tiles(&self, catalog: &Catalog, system: &SystemInfo, setup: &CurrentSetup<'_>) {
        let mut tiles = Vec::new();

        if let Some(distro) = system.current_distro(catalog) {
            tiles.push(tile(&distro.short, &distro.color, &distro.logo));
        }
        if let Some(desktop) = setup.desktop {
            tiles.push(tile(&desktop.short, &desktop.color, &desktop.logo));
        }
        if let Some(compositor) = setup.compositor {
            let already_shown = setup.desktop.is_some_and(|desktop| desktop.compositor == compositor.name);
            if !already_shown {
                tiles.push(tile(&compositor.short, &compositor.color, &compositor.logo));
            }
        }

        for content in tiles {
            self.imp()
                .tiles_box
                .append(&LogoTile::new(&content, SUMMARY_TILE_SIZE));
        }
    }

    fn show_facts(&self, catalog: &Catalog, system: &SystemInfo, setup: &CurrentSetup<'_>) {
        let hardware = &system.hardware;
        let unknown = gettext("Unknown");

        let facts = [
            fact(gettext("Distro"), non_empty(&system.os_release.pretty_name, &unknown)),
            fact(gettext("Desktop"), &system.desktop_name(catalog)),
            fact(
                gettext("Compositor"),
                setup.compositor.map_or(unknown.as_str(), |compositor| compositor.name.as_str()),
            ),
            fact(gettext("Kernel"), non_empty(&hardware.kernel, &unknown)),
            fact(gettext("Processor"), non_empty(&hardware.cpu_model, &unknown)),
            fact(gettext("Threads"), &hardware.cpu_threads.to_string()),
            fact(gettext("Memory"), &glib::format_size(hardware.memory_bytes)),
        ];

        for fact in &facts {
            self.imp().facts_flowbox.append(&fact_tile(fact));
        }
    }

    fn show_ideas(&self, catalog: &Catalog, setup: &CurrentSetup<'_>) {
        let imp = self.imp();
        let mut ready_flags = Vec::new();

        for idea in &catalog.ideas {
            let status = evaluate(idea, setup, catalog);
            ready_flags.push(status.is_ready());
            imp.ideas_flowbox.append(&IdeaCard::new(idea, &status));
        }

        let ready_count = ready_flags.iter().filter(|ready| **ready).count();
        self.show_ready_count(ready_count, ready_flags.len());
        self.show_ideas_note(setup);
        imp.ready_flags.replace(ready_flags);
    }

    fn show_ready_count(&self, ready: usize, total: usize) {
        let imp = self.imp();
        let share = if total == 0 { 0.0 } else { ready as f64 / total as f64 };

        imp.ready_bar.set_value(share);
        imp.ready_label.set_label(
            &gettext("{ready} of {total} ideas ready")
                .replace("{ready}", &ready.to_string())
                .replace("{total}", &total.to_string()),
        );
    }

    fn show_ideas_note(&self, setup: &CurrentSetup<'_>) {
        let note = match setup.compositor {
            Some(compositor) => gettext("Based on what {compositor} {version} offers, from wayland-explorer.")
                .replace("{compositor}", &compositor.name)
                .replace("{version}", &compositor.protocols.version),
            None => gettext("Linux & You could not tell which compositor you run, so only portal and desktop routes count."),
        };
        self.imp().ideas_note_label.set_label(&note);
    }

    fn show_visible_globals(&self, system: &SystemInfo) {
        let imp = self.imp();

        for global in &system.visible_globals {
            let chip = gtk::Label::new(Some(global));
            chip.add_css_class("protocol-chip");
            imp.visible_flowbox.append(&chip);
        }

        imp.visible_expander.set_label(Some(
            &gettext("Show {} protocols").replace("{}", &system.visible_globals.len().to_string()),
        ));
    }

    fn setup_status_filter(&self) {
        let imp = self.imp();

        imp.ideas_flowbox.set_filter_func(clone!(
            #[weak(rename_to = page)]
            self,
            #[upgrade_or]
            true,
            move |child| page.card_matches(child.index())
        ));

        imp.status_toggles.connect_active_name_notify(clone!(
            #[weak(rename_to = page)]
            self,
            move |_| page.imp().ideas_flowbox.invalidate_filter()
        ));
    }

    fn card_matches(&self, index: i32) -> bool {
        let imp = self.imp();
        let active = imp.status_toggles.active_name();
        let flags = imp.ready_flags.borrow();
        let Some(ready) = flags.get(index as usize) else {
            return true;
        };

        match active.as_deref() {
            None | Some(SHOW_ALL) => true,
            Some(SHOW_READY) => *ready,
            Some(_) => !*ready,
        }
    }
}

fn tile(short: &str, color: &str, logo: &Option<String>) -> TileContent {
    TileContent {
        short: short.to_owned(),
        color: color.to_owned(),
        logo: logo.clone(),
    }
}

fn fact(label: String, value: &str) -> Fact {
    Fact {
        label,
        value: value.to_owned(),
    }
}

fn non_empty<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.is_empty() { fallback } else { value }
}

/// For example "Wayland session".
fn session_text(session_type: &str) -> String {
    match session_type {
        "wayland" => gettext("Wayland session"),
        "x11" => gettext("X11 session"),
        "" => gettext("Unknown session"),
        other => gettext("{} session").replace("{}", other),
    }
}
