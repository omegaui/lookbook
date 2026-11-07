//! Your System: what the user runs, the stack it is built from, and which ideas
//! are ready on it.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::glib::clone;
use gtk::{gdk, glib};

use crate::catalog::Catalog;
use crate::ideas::{evaluate, CurrentSetup, IdeaStatus};
use crate::system::SystemInfo;
use crate::widgets::{fact_tile, fill_protocol_lists, Fact, IdeaCard, IsoStack, StackLayer};

const SHOW_ALL: &str = "all";
const SHOW_READY: &str = "ready";
const HARDWARE_COLOR: &str = "#9a9996";
const KERNEL_COLOR: &str = "#f6d32d";
const APPS_COLOR: &str = "#f6f5f4";
const FALLBACK_COLOR: &str = "#5e5c64";
const GLOW_COLOR: &str = "#3584e4";
const COPIED_RESET: Duration = Duration::from_secs(2);

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Lookbook/ui/pages/system_page.ui")]
    pub struct SystemPage {
        #[template_child]
        pub title_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub session_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub ready_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub ready_bar: gtk::TemplateChild<gtk::LevelBar>,
        #[template_child]
        pub copy_button: gtk::TemplateChild<gtk::Button>,
        #[template_child]
        pub stack: gtk::TemplateChild<IsoStack>,
        #[template_child]
        pub facts_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub ideas_note_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub status_toggles: gtk::TemplateChild<adw::ToggleGroup>,
        #[template_child]
        pub ideas_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub protocols_note_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub protocols_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub more_protocols_expander: gtk::TemplateChild<gtk::Expander>,
        #[template_child]
        pub more_protocols_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        pub catalog: RefCell<Option<Rc<Catalog>>>,
        /// Whether each card in `ideas_flowbox` is ready, in the same order.
        pub ready_flags: RefCell<Vec<bool>>,
        /// The text Copy Report puts on the clipboard.
        pub report: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SystemPage {
        const NAME: &'static str = "SystemPage";
        type Type = super::SystemPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            IsoStack::ensure_type();
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SystemPage {
        fn constructed(&self) {
            self.parent_constructed();
            let page = self.obj();
            page.setup_status_filter();
            page.make_ready_bar_plain();
        }
    }

    impl WidgetImpl for SystemPage {}
    impl BinImpl for SystemPage {}

    #[gtk::template_callbacks]
    impl SystemPage {
        #[template_callback]
        fn on_scan_clicked(&self, _button: &gtk::Button) {
            self.obj().scan_again();
        }

        #[template_callback]
        fn on_copy_clicked(&self, _button: &gtk::Button) {
            self.obj().copy_report();
        }
    }
}

glib::wrapper! {
    pub struct SystemPage(ObjectSubclass<imp::SystemPage>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl SystemPage {
    pub fn show_system(&self, catalog: Rc<Catalog>, system: &SystemInfo) {
        self.imp().catalog.replace(Some(catalog.clone()));
        let setup = CurrentSetup::detect(&catalog, system);
        let statuses: Vec<IdeaStatus> = catalog
            .ideas
            .iter()
            .map(|idea| evaluate(idea, &setup, &catalog))
            .collect();

        self.clear();
        self.show_summary(&catalog, system, &statuses);
        self.show_stack(&catalog, system, &setup);
        self.show_facts(&catalog, system, &setup);
        self.show_ideas(&catalog, &setup, &statuses);
        self.show_protocols(&catalog, system, &setup);
        self.prepare_report(&catalog, system, &statuses);
    }

    /// Detects everything again and redraws the page.
    fn scan_again(&self) {
        let catalog = self.imp().catalog.borrow().clone();
        if let Some(catalog) = catalog {
            self.show_system(catalog, &SystemInfo::detect());
        }
    }

    fn clear(&self) {
        let imp = self.imp();
        imp.facts_flowbox.remove_all();
        imp.ideas_flowbox.remove_all();
        imp.protocols_flowbox.remove_all();
        imp.more_protocols_flowbox.remove_all();
        imp.ready_flags.borrow_mut().clear();
    }

    fn show_summary(&self, catalog: &Catalog, system: &SystemInfo, statuses: &[IdeaStatus]) {
        let imp = self.imp();
        let ready = statuses.iter().filter(|status| status.is_available()).count();
        let total = statuses.len();
        let share = if total == 0 { 0.0 } else { ready as f64 / total as f64 };

        imp.title_label.set_label(&format!(
            "{} + {}",
            system.distro_name(),
            system.desktop_name(catalog)
        ));
        imp.session_label.set_label(&session_text(&system.session.session_type));
        imp.ready_label.set_label(
            &gettext("{ready} of {total}")
                .replace("{ready}", &ready.to_string())
                .replace("{total}", &total.to_string()),
        );
        imp.ready_bar.set_value(share);
    }

    /// Hardware, kernel, distro, desktop or compositor, and apps, from the bottom up.
    fn show_stack(&self, catalog: &Catalog, system: &SystemInfo, setup: &CurrentSetup<'_>) {
        let distro = system.current_distro(catalog);
        let mut layers = vec![
            layer(gettext("Hardware"), HARDWARE_COLOR, None),
            layer(kernel_label(&system.hardware.kernel), KERNEL_COLOR, None),
            layer(
                system.distro_name(),
                distro.map_or(FALLBACK_COLOR, |distro| distro.color.as_str()),
                distro.and_then(|distro| logo_icon(&distro.logo)),
            ),
        ];

        if let Some(desktop) = setup.desktop {
            layers.push(layer(desktop.name.clone(), &desktop.color, logo_icon(&desktop.logo)));
        } else if let Some(compositor) = setup.compositor {
            layers.push(layer(compositor.name.clone(), &compositor.color, logo_icon(&compositor.logo)));
        }

        let mut apps = layer(gettext("Your apps"), APPS_COLOR, None);
        apps.shows_apps = true;
        layers.push(apps);

        self.imp().stack.set_layers(layers, parse_color(GLOW_COLOR));
    }

    fn show_facts(&self, catalog: &Catalog, system: &SystemInfo, setup: &CurrentSetup<'_>) {
        let hardware = &system.hardware;
        let unknown = gettext("Unknown");
        let distro_detail = system
            .current_distro(catalog)
            .map(|distro| format!("{} · {}", distro.release.label(), distro.package_manager))
            .unwrap_or_default();
        let compositor_name = setup
            .compositor
            .map_or(unknown.clone(), |compositor| compositor.name.clone());
        let graphics = hardware.gpus.first().cloned().unwrap_or_else(|| unknown.clone());
        let other_graphics = hardware.gpus.iter().skip(1).cloned().collect::<Vec<_>>().join(", ");

        let facts = [
            Fact::with_detail(gettext("Distro"), non_empty(&system.os_release.pretty_name, &unknown), &distro_detail),
            Fact::with_detail(gettext("Session"), &session_name(&system.session.session_type), &compositor_name),
            Fact::with_detail(gettext("Kernel"), non_empty(&hardware.kernel, &unknown), std::env::consts::ARCH),
            Fact::with_detail(
                gettext("Processor"),
                non_empty(&hardware.cpu_model, &unknown),
                &gettext("{} threads").replace("{}", &hardware.cpu_threads.to_string()),
            ),
            Fact::new(gettext("Memory"), &glib::format_size(hardware.memory_bytes)),
            Fact::with_detail(gettext("Graphics"), &graphics, &other_graphics),
        ];

        for fact in &facts {
            self.imp().facts_flowbox.append(&fact_tile(fact));
        }
    }

    fn show_ideas(&self, catalog: &Catalog, setup: &CurrentSetup<'_>, statuses: &[IdeaStatus]) {
        let imp = self.imp();

        for (idea, status) in catalog.ideas.iter().zip(statuses) {
            imp.ideas_flowbox.append(&IdeaCard::new(idea, status));
        }
        imp.ready_flags
            .replace(statuses.iter().map(IdeaStatus::is_available).collect());

        let note = match setup.compositor {
            Some(compositor) => gettext("Built in means your desktop or compositor already includes it. An idea is ready to build when {compositor} {version} offers what it needs, from wayland-explorer data.")
                .replace("{compositor}", &compositor.name)
                .replace("{version}", &compositor.protocols.version),
            None => gettext("Lookbook could not tell which compositor you run, so only portal and desktop routes count."),
        };
        imp.ideas_note_label.set_label(&note);
    }

    /// Everything the compositor offers, with what each protocol is for.
    /// Protocols hidden from the sandbox are marked.
    fn show_protocols(&self, catalog: &Catalog, system: &SystemInfo, setup: &CurrentSetup<'_>) {
        let imp = self.imp();
        let globals = match setup.compositor {
            Some(compositor) => compositor.protocols.globals.clone(),
            None => system.visible_globals.clone(),
        };
        let overview = catalog.protocol_overview(&globals);

        fill_protocol_lists(
            &imp.protocols_flowbox,
            &imp.more_protocols_flowbox,
            &overview,
            Some(&system.visible_globals),
        );

        let compositor_name = setup
            .compositor
            .map_or(gettext("Your compositor"), |compositor| compositor.name.clone());
        imp.protocols_note_label.set_label(
            &gettext("{compositor} offers {total} protocols. Lookbook can see {visible} of them from inside its sandbox.")
                .replace("{compositor}", &compositor_name)
                .replace("{total}", &globals.len().to_string())
                .replace("{visible}", &system.visible_globals.len().to_string()),
        );
        imp.more_protocols_expander.set_label(Some(
            &gettext("{} more protocols").replace("{}", &overview.others.len().to_string()),
        ));
        imp.more_protocols_expander
            .set_visible(!overview.others.is_empty());
    }

    /// A plain-text summary someone can paste into a bug report or a forum post.
    fn prepare_report(&self, catalog: &Catalog, system: &SystemInfo, statuses: &[IdeaStatus]) {
        let hardware = &system.hardware;
        let mut lines = vec![
            "Lookbook report".to_owned(),
            format!("Distro: {}", system.os_release.pretty_name),
            format!("Desktop: {}", system.desktop_name(catalog)),
            format!("Session: {}", system.session.session_type),
            format!("Kernel: {}", hardware.kernel),
            format!("Processor: {} ({} threads)", hardware.cpu_model, hardware.cpu_threads),
            format!("Memory: {}", glib::format_size(hardware.memory_bytes)),
            format!("Graphics: {}", hardware.gpus.join(", ")),
            String::new(),
            "Ideas:".to_owned(),
        ];

        for (idea, status) in catalog.ideas.iter().zip(statuses) {
            let state = match status {
                IdeaStatus::BuiltIn { by, .. } => format!("built into {by}"),
                IdeaStatus::Ready { route, .. } => format!("ready ({route})"),
                IdeaStatus::WorksElsewhere { needs, .. } => format!("works elsewhere (needs {needs})"),
            };
            lines.push(format!("- {}: {state}", idea.name));
        }

        self.imp().report.replace(lines.join("\n"));
    }

    fn copy_report(&self) {
        let imp = self.imp();
        self.clipboard().set_text(&imp.report.borrow());
        imp.copy_button.set_label(&gettext("Copied"));

        glib::timeout_add_local_once(
            COPIED_RESET,
            clone!(
                #[weak(rename_to = page)]
                self,
                move || page.imp().copy_button.set_label(&gettext("Copy Report"))
            ),
        );
    }

    /// Without the default offsets, the bar keeps one color instead of turning
    /// red or orange at low values, which would read as a warning.
    fn make_ready_bar_plain(&self) {
        let bar = &self.imp().ready_bar;
        for offset in [gtk::LEVEL_BAR_OFFSET_LOW, gtk::LEVEL_BAR_OFFSET_HIGH, gtk::LEVEL_BAR_OFFSET_FULL] {
            bar.remove_offset_value(Some(offset));
        }
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

fn layer(label: String, color: &str, icon_name: Option<String>) -> StackLayer {
    StackLayer {
        label,
        color: parse_color(color),
        icon_name,
        shows_apps: false,
    }
}

fn logo_icon(logo: &Option<String>) -> Option<String> {
    logo.as_ref().map(|logo| format!("lay-logo-{logo}-symbolic"))
}

fn parse_color(color: &str) -> gdk::RGBA {
    gdk::RGBA::parse(color).unwrap_or(gdk::RGBA::BLACK)
}

/// "7.3.1-1-cachyos" becomes "Linux 7.3".
fn kernel_label(kernel: &str) -> String {
    let version: Vec<&str> = kernel.split(['.', '-']).take(2).collect();
    if version.len() < 2 {
        return gettext("Linux");
    }
    format!("Linux {}.{}", version[0], version[1])
}

fn non_empty<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.is_empty() { fallback } else { value }
}

fn session_name(session_type: &str) -> String {
    match session_type {
        "wayland" => gettext("Wayland"),
        "x11" => gettext("X11"),
        "" => gettext("Unknown"),
        other => other.to_owned(),
    }
}

/// The line under the title.
fn session_text(session_type: &str) -> String {
    let session = match session_type {
        "wayland" => gettext("Wayland session"),
        "x11" => gettext("X11 session"),
        _ => gettext("Session"),
    };
    gettext("{session}. Lookbook read your os-release file, your session variables and the protocols your compositor offers.")
        .replace("{session}", &session)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kernel_labels_keep_major_and_minor() {
        assert_eq!(kernel_label("7.3.1-1-cachyos"), "Linux 7.3");
        assert_eq!(kernel_label("6.17-arch1"), "Linux 6.17");
        assert_eq!(kernel_label(""), "Linux");
    }
}
