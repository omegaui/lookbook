//! "How Linux Fits Together": a dialog that draws a setup as layers people
//! can pick, and explains each one. It shows the user's own setup, a full
//! desktop with its compositor inside, and a compositor on its own.

use std::cell::{Cell, RefCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::glib::clone;

use super::{append_to_grid, LogoTile, TileContent};
use crate::catalog::{Catalog, ItemRef};
use crate::fits_together::{
    compositor_scene, explain, full_desktop_scene, your_scene, Explanation, Layer, LayerKind, Scene,
    SceneKind,
};
use crate::ideas::CurrentSetup;
use crate::system::SystemInfo;

const LAYER_ICON_SIZE: i32 = 28;
const EXAMPLE_TILE_SIZE: i32 = 36;
/// Each layer is this much narrower on each side than the one below it.
const NARROWING: i32 = 8;

/// An example item, ready to show: where it leads, its name and its tile.
#[derive(Debug, Clone)]
struct Example {
    item: ItemRef,
    name: String,
    tile: TileContent,
}

/// One layer as drawn: the layer, what to say about it, and its button.
#[derive(Debug, Clone)]
struct LayerView {
    layer: Layer,
    explanation: Explanation,
    examples: Vec<Example>,
    button: gtk::Button,
}

/// The drawn layers of one setup and which one is picked.
#[derive(Debug)]
struct SceneView {
    kind: SceneKind,
    layers: Vec<LayerView>,
    selected: Cell<usize>,
}

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/omegaui/Lookbook/ui/widgets/fits_together_dialog.ui")]
    pub struct FitsTogetherDialog {
        #[template_child]
        pub scene_toggles: gtk::TemplateChild<adw::ToggleGroup>,
        #[template_child]
        pub scene_stack: gtk::TemplateChild<gtk::Stack>,
        #[template_child]
        pub kind_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub name_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub what_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub change_label: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub examples_title: gtk::TemplateChild<gtk::Label>,
        #[template_child]
        pub examples_flowbox: gtk::TemplateChild<gtk::FlowBox>,
        pub(super) scenes: RefCell<Vec<SceneView>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FitsTogetherDialog {
        const NAME: &'static str = "FitsTogetherDialog";
        type Type = super::FitsTogetherDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FitsTogetherDialog {}
    impl WidgetImpl for FitsTogetherDialog {}
    impl AdwDialogImpl for FitsTogetherDialog {}
}

glib::wrapper! {
    pub struct FitsTogetherDialog(ObjectSubclass<imp::FitsTogetherDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl FitsTogetherDialog {
    pub fn new(catalog: &Catalog, system: &SystemInfo) -> Self {
        let dialog: Self = glib::Object::new();
        let setup = CurrentSetup::detect(catalog, system);

        dialog.add_scene(SceneKind::Yours, your_scene(catalog, system, &setup), catalog);
        dialog.add_scene(SceneKind::FullDesktop, full_desktop_scene(catalog), catalog);
        dialog.add_scene(SceneKind::CompositorOnly, compositor_scene(catalog), catalog);
        dialog.follow_scene_toggles();
        dialog.show_scene(SceneKind::Yours);
        dialog
    }

    /// Shows a setup with one of its layers picked, counting from the top.
    pub fn show_layer(&self, kind: SceneKind, index: usize) {
        self.imp().scene_toggles.set_active_name(Some(kind.id()));
        self.select(kind, index);
    }

    /// Draws a setup's layers and remembers them, so picking one can explain it.
    fn add_scene(&self, kind: SceneKind, scene: Scene, catalog: &Catalog) {
        let layers: Vec<LayerView> = scene
            .layers
            .iter()
            .map(|layer| self.layer_view(kind, layer, catalog))
            .collect();

        let drawing = layers_drawing(&layers);
        self.imp().scene_stack.add_named(&drawing, Some(kind.id()));
        self.imp().scenes.borrow_mut().push(SceneView {
            kind,
            layers,
            selected: Cell::new(scene.first_selected()),
        });
    }

    fn layer_view(&self, kind: SceneKind, layer: &Layer, catalog: &Catalog) -> LayerView {
        let explanation = explain(layer);
        let examples = explanation
            .examples
            .iter()
            .filter_map(|item| example(item, catalog))
            .collect();
        let button = layer_button(layer);
        self.select_on_click(&button, kind);

        LayerView {
            layer: layer.clone(),
            explanation,
            examples,
            button,
        }
    }

    fn select_on_click(&self, button: &gtk::Button, kind: SceneKind) {
        button.connect_clicked(clone!(
            #[weak(rename_to = dialog)]
            self,
            move |button| dialog.select_button(kind, button)
        ));
    }

    fn follow_scene_toggles(&self) {
        self.imp().scene_toggles.connect_active_name_notify(clone!(
            #[weak(rename_to = dialog)]
            self,
            move |toggles| {
                let kind = toggles.active_name().as_deref().and_then(SceneKind::from_id);
                if let Some(kind) = kind {
                    dialog.show_scene(kind);
                }
            }
        ));
    }

    fn show_scene(&self, kind: SceneKind) {
        self.imp().scene_stack.set_visible_child_name(kind.id());
        let selected = self.with_scene(kind, |scene| scene.selected.get());
        if let Some(index) = selected {
            self.select(kind, index);
        }
    }

    fn select_button(&self, kind: SceneKind, button: &gtk::Button) {
        let index = self.with_scene(kind, |scene| {
            scene.layers.iter().position(|view| &view.button == button)
        });
        if let Some(Some(index)) = index {
            self.select(kind, index);
        }
    }

    /// Marks the picked layer and explains it.
    fn select(&self, kind: SceneKind, index: usize) {
        let picked = self.with_scene(kind, |scene| {
            scene.selected.set(index);
            for (position, view) in scene.layers.iter().enumerate() {
                set_picked(&view.button, position == index);
            }
            scene.layers.get(index).cloned()
        });
        if let Some(Some(view)) = picked {
            self.show_details(&view);
        }
    }

    fn with_scene<T>(&self, kind: SceneKind, action: impl FnOnce(&SceneView) -> T) -> Option<T> {
        let scenes = self.imp().scenes.borrow();
        scenes.iter().find(|scene| scene.kind == kind).map(action)
    }

    fn show_details(&self, view: &LayerView) {
        let imp = self.imp();
        imp.kind_label.set_label(&view.layer.kind.title());
        imp.name_label.set_label(&view.layer.name);
        imp.what_label.set_label(&view.explanation.what);
        imp.change_label.set_label(&view.explanation.change);
        self.show_examples(&view.examples);
    }

    fn show_examples(&self, examples: &[Example]) {
        let imp = self.imp();
        imp.examples_flowbox.remove_all();
        for example in examples {
            append_to_grid(&imp.examples_flowbox, &self.example_button(example));
        }
        imp.examples_title.set_visible(!examples.is_empty());
        imp.examples_flowbox.set_visible(!examples.is_empty());
    }

    /// Opens the example's details page, and closes the dialog so it shows.
    fn example_button(&self, example: &Example) -> gtk::Button {
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        content.append(&LogoTile::new(&example.tile, EXAMPLE_TILE_SIZE));
        let name = gtk::Label::new(Some(&example.name));
        name.set_xalign(0.0);
        name.set_ellipsize(gtk::pango::EllipsizeMode::End);
        content.append(&name);

        let button = gtk::Button::builder()
            .child(&content)
            .action_name("win.show-item")
            .action_target(&example.item.to_action_target().to_variant())
            .css_classes(["fit-example"])
            .build();
        button.connect_clicked(clone!(
            #[weak(rename_to = dialog)]
            self,
            move |_| {
                dialog.close();
            }
        ));
        button
    }
}

fn example(item: &ItemRef, catalog: &Catalog) -> Option<Example> {
    let summary = catalog.summary(item.kind, &item.id)?;
    Some(Example {
        item: item.clone(),
        name: summary.name.to_owned(),
        tile: TileContent {
            short: summary.short.to_owned(),
            color: summary.color.to_owned(),
            logo: summary.logo.map(str::to_owned),
            icon_name: None,
        },
    })
}

/// The layers from top to bottom, each a little wider than the one above.
/// A desktop and its own compositor share one row, the compositor inside.
fn layers_drawing(layers: &[LayerView]) -> gtk::Box {
    let drawing = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let mut index = 0;
    while index < layers.len() {
        let view = &layers[index];
        let next = layers.get(index + 1);
        let row = match next {
            Some(inner) if view.layer.kind == LayerKind::Desktop && inner.layer.built_into.is_some() => {
                index += 1;
                desktop_row(&view.button, &inner.button)
            }
            _ => view.button.clone().upcast(),
        };
        drawing.append(&row);
        index += 1;
    }
    narrow_toward_the_top(&drawing);
    drawing
}

/// A desktop's panel, with its compositor as a smaller block inside it.
pub fn desktop_row(desktop: &impl IsA<gtk::Widget>, compositor: &impl IsA<gtk::Widget>) -> gtk::Widget {
    let row = gtk::Box::new(gtk::Orientation::Vertical, 6);
    row.add_css_class("fit-desktop-row");
    row.append(desktop);
    compositor.add_css_class("fit-inside");
    row.append(compositor);
    row.upcast()
}

fn narrow_toward_the_top(drawing: &gtk::Box) {
    let rows: Vec<gtk::Widget> = std::iter::successors(drawing.first_child(), |row| row.next_sibling()).collect();
    let count = rows.len() as i32;
    for (position, row) in rows.iter().enumerate() {
        let from_bottom = count - 1 - position as i32;
        row.set_margin_start(from_bottom * NARROWING);
        row.set_margin_end(from_bottom * NARROWING);
    }
}

/// A layer's button: its logo or icon, what kind of layer it is, and its name.
fn layer_button(layer: &Layer) -> gtk::Button {
    let button = gtk::Button::builder()
        .child(&layer_content(layer))
        .css_classes(["fit-layer", layer.kind.css_class()])
        .build();
    button.update_property(&[gtk::accessible::Property::Label(&format!(
        "{}: {}",
        layer.kind.title(),
        layer.name
    ))]);
    button
}

/// The same layer as a plain block, for drawings nobody clicks.
pub fn layer_block(layer: &Layer) -> gtk::Box {
    let block = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    block.append(&layer_content(layer));
    block.set_css_classes(&["fit-layer", layer.kind.css_class()]);
    block
}

fn layer_content(layer: &Layer) -> gtk::Box {
    let icon = gtk::Image::from_icon_name(&layer_icon(layer));
    icon.set_pixel_size(LAYER_ICON_SIZE);

    let kind = gtk::Label::new(Some(&layer.kind.title()));
    kind.add_css_class("fit-layer-kind");
    kind.set_xalign(0.0);
    let name = gtk::Label::new(Some(&layer.name));
    name.add_css_class("heading");
    name.set_xalign(0.0);
    name.set_wrap(true);

    let texts = gtk::Box::new(gtk::Orientation::Vertical, 0);
    texts.set_valign(gtk::Align::Center);
    texts.append(&kind);
    texts.append(&name);

    let content = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    content.append(&icon);
    content.append(&texts);
    content
}

fn layer_icon(layer: &Layer) -> String {
    if let Some(logo) = &layer.logo {
        return format!("lay-logo-{logo}-symbolic");
    }
    match layer.kind {
        LayerKind::Apps => "view-app-grid-symbolic",
        LayerKind::Pieces => "list-add-symbolic",
        LayerKind::Kernel => "lay-logo-linux-symbolic",
        _ => "computer-symbolic",
    }
    .to_owned()
}

fn set_picked(button: &gtk::Button, picked: bool) {
    if picked {
        button.add_css_class("picked");
    } else {
        button.remove_css_class("picked");
    }
}
