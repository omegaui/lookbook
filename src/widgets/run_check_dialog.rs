//! The "Can I Run It?" sheet: a verdict, what the user's computer means for the
//! item, whether their distro packages it (asked live from Repology), the ideas
//! they would gain or lose, and how to try it.

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::glib;
use gtk::glib::clone;

use super::{open_link, LogoTile, TileContent};
use crate::http::new_session;
use crate::repology::{fetch_project, find_package, PackageMatch};
use crate::run_check::{CheckLine, CheckState, PackageLookup, RunCheck, TryStep};

const TILE_SIZE: i32 = 72;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/LinuxAndYou/ui/widgets/run_check_dialog.ui")]
    pub struct RunCheckDialog {
        #[template_child]
        pub tile: gtk::TemplateChild<LogoTile>,
        #[template_child]
        pub question_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub verdict_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub verdict_detail_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub checks_group: gtk::TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub package_group: gtk::TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub ideas_group: gtk::TemplateChild<adw::PreferencesGroup>,
        #[template_child]
        pub try_group: gtk::TemplateChild<adw::PreferencesGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RunCheckDialog {
        const NAME: &'static str = "RunCheckDialog";
        type Type = super::RunCheckDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            LogoTile::ensure_type();
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RunCheckDialog {}
    impl WidgetImpl for RunCheckDialog {}
    impl AdwDialogImpl for RunCheckDialog {}
}

glib::wrapper! {
    pub struct RunCheckDialog(ObjectSubclass<imp::RunCheckDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::ShortcutManager;
}

impl RunCheckDialog {
    pub fn new(result: &RunCheck, tile: &TileContent) -> Self {
        let dialog: Self = glib::Object::builder()
            .property("title", gettext("Can I Run It?"))
            .build();

        dialog.show_verdict(result, tile);
        dialog.show_checks(&result.checks);
        dialog.show_idea_changes(result);
        dialog.show_try_steps(&result.try_steps);
        dialog.look_up_package(result);
        dialog
    }

    fn show_verdict(&self, result: &RunCheck, tile: &TileContent) {
        let imp = self.imp();
        imp.tile.set_tile_size(TILE_SIZE);
        imp.tile.set_content(tile);
        imp.question_label
            .set_label(&gettext("Can I run {}?").replace("{}", &result.name));
        imp.verdict_label.set_label(&result.verdict);
        imp.verdict_detail_label.set_label(&result.verdict_detail);
    }

    fn show_checks(&self, checks: &[CheckLine]) {
        for check in checks {
            let row = adw::ActionRow::builder()
                .title(glib::markup_escape_text(&check.title))
                .subtitle(glib::markup_escape_text(&check.detail))
                .build();
            row.add_prefix(&state_icon(check.state));
            self.imp().checks_group.add(&row);
        }
    }

    /// Two rows, one per direction, listing idea names, or one row saying nothing changes.
    fn show_idea_changes(&self, result: &RunCheck) {
        let group = &self.imp().ideas_group;
        if result.gained.is_empty() && result.lost.is_empty() && !result.is_running {
            let row = adw::ActionRow::builder()
                .title(gettext("Nothing to give up"))
                .subtitle(gettext("Every idea you have now is built in or ready to build there too."))
                .build();
            row.add_prefix(&state_icon(CheckState::Good));
            group.add(&row);
            group.set_visible(true);
            return;
        }

        let rows = [
            (gettext("You gain"), &result.gained, "list-add-symbolic"),
            (gettext("You give up"), &result.lost, "list-remove-symbolic"),
        ];

        for (title, ideas, icon_name) in rows {
            if ideas.is_empty() {
                continue;
            }
            let row = adw::ActionRow::builder()
                .title(title)
                .subtitle(glib::markup_escape_text(&ideas.join(", ")))
                .build();
            row.add_prefix(&gtk::Image::from_icon_name(icon_name));
            group.add(&row);
            group.set_visible(true);
        }
    }

    fn show_try_steps(&self, steps: &[TryStep]) {
        for step in steps {
            let row = adw::ActionRow::builder()
                .title(glib::markup_escape_text(&step.title))
                .subtitle(glib::markup_escape_text(&step.detail))
                .build();
            if let Some(link) = &step.link {
                row.add_suffix(&link_button(&link.label, &link.url));
            }
            self.imp().try_group.add(&row);
        }
    }

    /// Asks Repology in the background; the Packages group fills in when it answers.
    fn look_up_package(&self, result: &RunCheck) {
        let Some(lookup) = result.package.clone() else {
            return;
        };
        let is_running = result.is_running;
        let pending_row = self.show_pending_package(&lookup);

        glib::spawn_future_local(clone!(
            #[weak(rename_to = dialog)]
            self,
            async move {
                let found = dialog.fetch_package(&lookup).await;
                dialog.imp().package_group.remove(&pending_row);
                dialog.show_package(&lookup, found, is_running);
            }
        ));
    }

    fn show_pending_package(&self, lookup: &PackageLookup) -> adw::ActionRow {
        let group = &self.imp().package_group;
        let row = adw::ActionRow::builder()
            .title(gettext("Looking in {}'s repositories…").replace("{}", &lookup.distro_name))
            .build();
        row.add_suffix(&adw::Spinner::new());
        group.add(&row);
        group.set_visible(true);
        row
    }

    async fn fetch_package(&self, lookup: &PackageLookup) -> Result<Option<PackageMatch>, ()> {
        let session = new_session();
        match fetch_project(&session, &lookup.project).await {
            Ok(packages) => Ok(find_package(&packages, &lookup.repos, &lookup.project)),
            Err(error) => {
                tracing::warn!("Could not ask Repology about {}: {error:#}", lookup.project);
                Err(())
            }
        }
    }

    fn show_package(&self, lookup: &PackageLookup, found: Result<Option<PackageMatch>, ()>, is_running: bool) {
        match found {
            Ok(Some(package)) => self.show_found_package(lookup, &package, is_running),
            Ok(None) => self.show_missing_package(lookup, is_running),
            Err(()) => self.show_package_error(),
        }
    }

    fn show_found_package(&self, lookup: &PackageLookup, package: &PackageMatch, is_running: bool) {
        let imp = self.imp();
        let where_found = match &package.repo_label {
            Some(label) => format!("{} ({label})", lookup.distro_name),
            None => lookup.distro_name.clone(),
        };
        let title = gettext("{package} {version} in {distro}")
            .replace("{package}", &package.name)
            .replace("{version}", &package.version)
            .replace("{distro}", &where_found);

        let row = adw::ActionRow::builder()
            .title(glib::markup_escape_text(&title))
            .subtitle(glib::markup_escape_text(&package.install_command))
            .subtitle_selectable(true)
            .build();
        row.add_css_class("property");
        row.add_prefix(&state_icon(CheckState::Good));
        row.add_suffix(&copy_button(&package.install_command));
        imp.package_group.add(&row);

        if lookup.is_desktop {
            imp.package_group.set_description(Some(&gettext(
                "Desktops often install as a group or meta-package; your distro's wiki names it. Package data from Repology.",
            )));
        }
        if !is_running {
            imp.verdict_label.set_label(&gettext("Yes, your distro packages it."));
        }
    }

    fn show_missing_package(&self, lookup: &PackageLookup, is_running: bool) {
        let imp = self.imp();
        let row = adw::ActionRow::builder()
            .title(gettext("Not in {}'s repositories").replace("{}", &lookup.distro_name))
            .subtitle(gettext("It may still build from source, or come from a community repository. See How to Try It."))
            .build();
        row.add_prefix(&state_icon(CheckState::Note));
        imp.package_group.add(&row);

        if !is_running {
            imp.verdict_label.set_label(&gettext("Possibly, with some extra steps."));
        }
    }

    fn show_package_error(&self) {
        let imp = self.imp();
        let row = adw::ActionRow::builder()
            .title(gettext("Repology could not be reached"))
            .subtitle(gettext("Check your connection, then open this sheet again."))
            .build();
        row.add_prefix(&state_icon(CheckState::Problem));
        imp.package_group.add(&row);
        imp.verdict_label.set_label(&gettext("Could not check your distro's packages."));
    }
}

/// A yellow tick, an info sign or a warning sign.
fn state_icon(state: CheckState) -> gtk::Image {
    let (icon_name, css_class) = match state {
        CheckState::Good => ("object-select-symbolic", "check-good"),
        CheckState::Note => ("dialog-information-symbolic", "check-note"),
        CheckState::Problem => ("dialog-warning-symbolic", "check-problem"),
    };
    let icon = gtk::Image::from_icon_name(icon_name);
    icon.add_css_class(css_class);
    icon.set_accessible_role(gtk::AccessibleRole::Presentation);
    icon
}

fn link_button(label: &str, url: &str) -> gtk::Button {
    let button = gtk::Button::builder()
        .label(label)
        .valign(gtk::Align::Center)
        .css_classes(["flat"])
        .build();
    let url = url.to_owned();
    button.connect_clicked(move |button| open_link(button, &url));
    button
}

fn copy_button(text: &str) -> gtk::Button {
    let button = gtk::Button::builder()
        .icon_name("edit-copy-symbolic")
        .tooltip_text(gettext("Copy"))
        .valign(gtk::Align::Center)
        .css_classes(["flat"])
        .build();
    let text = text.to_owned();
    button.connect_clicked(move |button| {
        button.clipboard().set_text(&text);
        button.set_icon_name("object-select-symbolic");
    });
    button
}
