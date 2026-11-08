//! The hero at the top of the Distros, Desktops and Compositors pages: what
//! the thing is, the mix-up people make about it, and a small drawing made of
//! the same layer blocks as "How Linux Fits Together".

use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::glib;

use super::{desktop_row, layer_block};
use crate::fits_together::{IntroDiagram, Layer, PageIntro};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Syspedia/ui/widgets/intro_banner.ui")]
    pub struct IntroBanner {
        #[template_child]
        pub kicker_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub title_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub body_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub mix_up_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub explain_button: gtk::TemplateChild<gtk::Button>,
        #[template_child]
        pub diagram_box: gtk::TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IntroBanner {
        const NAME: &'static str = "IntroBanner";
        type Type = super::IntroBanner;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for IntroBanner {}
    impl WidgetImpl for IntroBanner {}
    impl BinImpl for IntroBanner {}
}

glib::wrapper! {
    pub struct IntroBanner(ObjectSubclass<imp::IntroBanner>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl IntroBanner {
    pub fn show(&self, intro: &PageIntro) {
        let imp = self.imp();
        imp.kicker_label.set_label(&intro.kicker);
        imp.title_label.set_label(&intro.title);
        imp.body_label.set_label(&intro.body);
        imp.mix_up_label.set_label(&intro.mix_up);
        imp.explain_button
            .set_action_target_value(Some(&intro.explain_target.to_variant()));
        self.show_diagram(&intro.diagram);
    }

    fn show_diagram(&self, diagram: &IntroDiagram) {
        let diagram_box = &self.imp().diagram_box;
        while let Some(child) = diagram_box.first_child() {
            diagram_box.remove(&child);
        }
        let drawing = match diagram {
            IntroDiagram::OneDistroManyLooks { desktops, distro } => one_distro_many_looks(desktops, distro),
            IntroDiagram::DesktopBringsCompositor { desktop, compositor, distro } => {
                desktop_brings_compositor(desktop, compositor, distro)
            }
            IntroDiagram::InsideOrAlone { desktop, built_in, pieces, alone } => {
                inside_or_alone(desktop, built_in, pieces, alone)
            }
        };
        diagram_box.append(&drawing);
    }
}

/// Three desktops side by side on one distro, with a caption.
fn one_distro_many_looks(desktops: &[Layer], distro: &Layer) -> gtk::Box {
    let looks = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    looks.set_homogeneous(true);
    for desktop in desktops {
        looks.append(&layer_chip(desktop));
    }

    let drawing = column(&[looks.upcast(), layer_block(distro).upcast()]);
    drawing.append(&caption(&gettext("One distro, three different looks")));
    drawing
}

/// A desktop panel with its compositor inside, over a faded "any distro".
fn desktop_brings_compositor(desktop: &Layer, compositor: &Layer, distro: &Layer) -> gtk::Box {
    let base = layer_block(distro);
    base.add_css_class("fit-faded");

    let drawing = column(&[desktop_row(&layer_block(desktop), &layer_block(compositor)), base.upcast()]);
    drawing.append(&caption(&gettext("The compositor comes inside the desktop")));
    drawing
}

/// Two columns: a compositor inside a desktop, and one on its own with the
/// pieces you add yourself.
fn inside_or_alone(desktop: &Layer, built_in: &Layer, pieces: &Layer, alone: &Layer) -> gtk::Box {
    let inside = column(&[
        caption(&gettext("Inside a desktop")).upcast(),
        desktop_row(&layer_block(desktop), &layer_block(built_in)),
    ]);
    let on_its_own = column(&[
        caption(&gettext("On its own")).upcast(),
        layer_block(pieces).upcast(),
        layer_block(alone).upcast(),
    ]);
    inside.set_valign(gtk::Align::End);
    on_its_own.set_valign(gtk::Align::End);

    let drawing = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    drawing.set_homogeneous(true);
    drawing.append(&inside);
    drawing.append(&on_its_own);
    drawing
}

fn column(children: &[gtk::Widget]) -> gtk::Box {
    let column = gtk::Box::new(gtk::Orientation::Vertical, 8);
    for child in children {
        column.append(child);
    }
    column
}

/// A small block with the logo above the name, for rows of several layers.
fn layer_chip(layer: &Layer) -> gtk::Box {
    let chip = gtk::Box::new(gtk::Orientation::Vertical, 6);
    chip.set_css_classes(&["fit-layer", "fit-chip", layer.kind.css_class()]);

    let icon_name = layer
        .logo
        .as_ref()
        .map_or_else(|| "computer-symbolic".to_owned(), |logo| format!("lay-logo-{logo}-symbolic"));
    let icon = gtk::Image::from_icon_name(&icon_name);
    icon.set_pixel_size(28);
    chip.append(&icon);

    let name = gtk::Label::new(Some(&layer.name));
    name.add_css_class("heading");
    name.set_wrap(true);
    name.set_justify(gtk::Justification::Center);
    chip.append(&name);
    chip
}

fn caption(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_css_classes(&["caption", "dim-label"]);
    label.set_wrap(true);
    label
}
