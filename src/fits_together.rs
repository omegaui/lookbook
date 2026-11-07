//! "How Linux fits together": the layers of a Linux system, for the
//! explainer on Your System. Most people start on Ubuntu or Fedora with GNOME
//! or KDE Plasma and never meet the word "compositor", so the explainer puts a
//! full desktop (with its compositor inside) next to a compositor on its own.
//! No UI code.

use gettextrs::gettext;

use crate::catalog::{Catalog, ItemKind, ItemRef};
use crate::ideas::CurrentSetup;
use crate::system::{kernel_label, SystemInfo};

/// The three setups the explainer can show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneKind {
    Yours,
    FullDesktop,
    CompositorOnly,
}

impl SceneKind {
    /// Matches the toggle names in the explainer's Blueprint file.
    pub fn id(self) -> &'static str {
        match self {
            Self::Yours => "yours",
            Self::FullDesktop => "desktop",
            Self::CompositorOnly => "compositor",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        [Self::Yours, Self::FullDesktop, Self::CompositorOnly]
            .into_iter()
            .find(|kind| kind.id() == id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerKind {
    Apps,
    /// The bar, launcher and the rest that a compositor on its own leaves to you.
    Pieces,
    Desktop,
    Compositor,
    Distro,
    Kernel,
}

impl LayerKind {
    /// What the layer is called, shown above its name.
    pub fn title(self) -> String {
        match self {
            Self::Apps => gettext("Apps"),
            Self::Pieces => gettext("You add"),
            Self::Desktop => gettext("Desktop"),
            Self::Compositor => gettext("Compositor"),
            Self::Distro => gettext("Distro"),
            Self::Kernel => gettext("Kernel"),
        }
    }

    /// Each kind has its own color in the stylesheet.
    pub fn css_class(self) -> &'static str {
        match self {
            Self::Apps => "fit-apps",
            Self::Pieces => "fit-pieces",
            Self::Desktop => "fit-desktop",
            Self::Compositor => "fit-compositor",
            Self::Distro => "fit-distro",
            Self::Kernel => "fit-kernel",
        }
    }
}

/// One layer of a setup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layer {
    pub kind: LayerKind,
    /// For example "Fedora Workstation" or "Mutter".
    pub name: String,
    /// A bundled logo id, for example "fedora".
    pub logo: Option<String>,
    /// For a compositor inside a desktop: that desktop's name, for example "GNOME".
    pub built_into: Option<String>,
}

impl Layer {
    fn new(kind: LayerKind, name: &str, logo: Option<&str>) -> Self {
        Self {
            kind,
            name: name.to_owned(),
            logo: logo.map(str::to_owned),
            built_into: None,
        }
    }
}

/// A setup from the top (apps) down to the kernel. A desktop's own
/// compositor follows the desktop, marked with `built_into`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scene {
    pub layers: Vec<Layer>,
}

impl Scene {
    /// The layer to explain first: the compositor, the part most people
    /// have never heard of.
    pub fn first_selected(&self) -> usize {
        self.layers
            .iter()
            .position(|layer| layer.kind == LayerKind::Compositor)
            .unwrap_or(0)
    }
}

/// What the details panel says about a layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explanation {
    pub what: String,
    pub change: String,
    /// Catalog items to show as examples.
    pub examples: Vec<ItemRef>,
}

/// The user's own setup, as far as the app could detect it.
pub fn your_scene(catalog: &Catalog, system: &SystemInfo, setup: &CurrentSetup<'_>) -> Scene {
    let mut layers = vec![apps_layer()];
    layers.extend(session_layers(catalog, system, setup));
    layers.push(your_distro_layer(catalog, system));
    layers.push(Layer::new(LayerKind::Kernel, &kernel_label(&system.hardware.kernel), Some("linux")));
    Scene { layers }
}

/// What most people start with: Fedora and GNOME, with Mutter inside GNOME.
pub fn full_desktop_scene(catalog: &Catalog) -> Scene {
    let mut layers = vec![apps_layer()];
    layers.extend(desktop_layers(catalog, "gnome"));
    layers.push(distro_layer(catalog, "fedora"));
    layers.push(Layer::new(LayerKind::Kernel, &gettext("Linux"), Some("linux")));
    Scene { layers }
}

/// A compositor on its own: Arch and Hyprland, with the pieces you add.
pub fn compositor_scene(catalog: &Catalog) -> Scene {
    let mut layers = vec![apps_layer(), pieces_layer()];
    layers.extend(compositor_layer(catalog, "hyprland"));
    layers.push(distro_layer(catalog, "arch"));
    layers.push(Layer::new(LayerKind::Kernel, &gettext("Linux"), Some("linux")));
    Scene { layers }
}

pub fn explain(layer: &Layer) -> Explanation {
    match layer.kind {
        LayerKind::Apps => explain_apps(),
        LayerKind::Pieces => explain_pieces(),
        LayerKind::Desktop => explain_desktop(),
        LayerKind::Compositor => match &layer.built_into {
            Some(desktop) => explain_built_in_compositor(&layer.name, desktop),
            None => explain_compositor_alone(),
        },
        LayerKind::Distro => explain_distro(),
        LayerKind::Kernel => explain_kernel(),
    }
}

fn apps_layer() -> Layer {
    Layer::new(LayerKind::Apps, &gettext("Firefox, Files, games…"), None)
}

fn pieces_layer() -> Layer {
    Layer::new(LayerKind::Pieces, &gettext("Bar, launcher, notifications"), None)
}

/// A desktop and its compositor, or a compositor on its own with the pieces
/// you add, or just a name when the session is not in the catalog.
fn session_layers(catalog: &Catalog, system: &SystemInfo, setup: &CurrentSetup<'_>) -> Vec<Layer> {
    if let Some(desktop) = setup.desktop {
        return desktop_layers(catalog, &desktop.id);
    }
    if let Some(compositor) = setup.compositor {
        let mut layers = vec![pieces_layer()];
        layers.extend(compositor_layer(catalog, &compositor.id));
        return layers;
    }
    vec![Layer::new(LayerKind::Desktop, &system.desktop_name(catalog), None)]
}

fn desktop_layers(catalog: &Catalog, desktop_id: &str) -> Vec<Layer> {
    let Some(desktop) = catalog.find_desktop(desktop_id) else {
        return Vec::new();
    };
    let compositor_logo = catalog
        .compositors
        .iter()
        .find(|compositor| compositor.name == desktop.compositor)
        .and_then(|compositor| compositor.logo.as_deref());
    let mut built_in = Layer::new(LayerKind::Compositor, &desktop.compositor, compositor_logo);
    built_in.built_into = Some(desktop.name.clone());

    vec![
        Layer::new(LayerKind::Desktop, &desktop.name, desktop.logo.as_deref()),
        built_in,
    ]
}

fn compositor_layer(catalog: &Catalog, compositor_id: &str) -> Option<Layer> {
    let compositor = catalog.find_compositor(compositor_id)?;
    Some(Layer::new(LayerKind::Compositor, &compositor.name, compositor.logo.as_deref()))
}

fn distro_layer(catalog: &Catalog, distro_id: &str) -> Layer {
    match catalog.find_distro(distro_id) {
        Some(distro) => Layer::new(LayerKind::Distro, &distro.name, distro.logo.as_deref()),
        None => Layer::new(LayerKind::Distro, distro_id, None),
    }
}

fn your_distro_layer(catalog: &Catalog, system: &SystemInfo) -> Layer {
    let logo = system
        .current_distro(catalog)
        .and_then(|distro| distro.logo.as_deref());
    Layer::new(LayerKind::Distro, &system.distro_name(), logo)
}

fn explain_apps() -> Explanation {
    Explanation {
        what: gettext("The programs you use every day. Most of them run on any distro and any desktop, especially apps from Flathub."),
        change: gettext("Install and remove them from your software center whenever you like."),
        examples: Vec::new(),
    }
}

fn explain_pieces() -> Explanation {
    Explanation {
        what: gettext("On its own, a compositor only draws windows. You choose a bar, an app launcher, notifications and a lock screen yourself, or start from a rice that bundles them."),
        change: gettext("Swap any piece at any time. That freedom is why people rice."),
        examples: Vec::new(),
    }
}

fn explain_desktop() -> Explanation {
    Explanation {
        what: gettext("What you see and touch: the top bar or panel, the app menu, settings and notifications. It comes with its own compositor, so you never set one up."),
        change: gettext("Install another desktop and pick it on the login screen. You can keep several side by side."),
        examples: items(ItemKind::Desktop, &["gnome", "kde-plasma", "cosmic", "cinnamon"]),
    }
}

fn explain_built_in_compositor(name: &str, desktop: &str) -> Explanation {
    Explanation {
        what: gettext("Draws every window on the screen and decides where it goes, how it moves and how it animates. In {desktop}, that job is done by {name}, built in.")
            .replace("{desktop}", desktop)
            .replace("{name}", name),
        change: gettext("It comes with the desktop. Changing the desktop changes the compositor too."),
        examples: items(ItemKind::Compositor, &["mutter", "kwin", "cosmic-comp"]),
    }
}

fn explain_compositor_alone() -> Explanation {
    Explanation {
        what: gettext("Draws every window on the screen and decides where it goes. Some compositors run on their own, without a desktop around them, and tile windows so you never drag them."),
        change: gettext("Install one next to your desktop and pick it on the login screen."),
        examples: items(ItemKind::Compositor, &["hyprland", "niri", "sway"]),
    }
}

fn explain_distro() -> Explanation {
    Explanation {
        what: gettext("What you install. It bundles the kernel, a desktop to start with, the apps, and decides how updates arrive."),
        change: gettext("Switching distro means installing a new one. Keep your files on a backup or a separate drive."),
        examples: items(ItemKind::Distro, &["fedora", "ubuntu", "linuxmint", "arch"]),
    }
}

fn explain_kernel() -> Explanation {
    Explanation {
        what: gettext("The core that talks to your hardware. Every distro here uses the same Linux kernel; that is what makes them all Linux."),
        change: gettext("Your distro updates it for you."),
        examples: Vec::new(),
    }
}

fn items(kind: ItemKind, ids: &[&str]) -> Vec<ItemRef> {
    ids.iter().map(|id| ItemRef::new(kind, id)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests have no GResource, so read the catalog file itself.
    fn catalog() -> Catalog {
        let json = include_bytes!("../data/resources/catalog/catalog.json");
        Catalog::from_json(json).expect("the bundled catalog should parse")
    }

    fn kinds(scene: &Scene) -> Vec<LayerKind> {
        scene.layers.iter().map(|layer| layer.kind).collect()
    }

    #[test]
    fn a_full_desktop_has_its_compositor_inside() {
        let scene = full_desktop_scene(&catalog());
        assert_eq!(
            kinds(&scene),
            [LayerKind::Apps, LayerKind::Desktop, LayerKind::Compositor, LayerKind::Distro, LayerKind::Kernel]
        );
        let compositor = &scene.layers[2];
        assert_eq!(compositor.name, "Mutter");
        assert_eq!(compositor.built_into.as_deref(), Some("GNOME"));
    }

    #[test]
    fn a_compositor_alone_leaves_pieces_to_you() {
        let scene = compositor_scene(&catalog());
        assert_eq!(
            kinds(&scene),
            [LayerKind::Apps, LayerKind::Pieces, LayerKind::Compositor, LayerKind::Distro, LayerKind::Kernel]
        );
        assert_eq!(scene.layers[2].built_into, None);
    }

    #[test]
    fn the_compositor_is_explained_first() {
        let scene = full_desktop_scene(&catalog());
        assert_eq!(scene.layers[scene.first_selected()].kind, LayerKind::Compositor);
    }

    #[test]
    fn a_built_in_compositor_names_its_desktop() {
        let scene = full_desktop_scene(&catalog());
        let explanation = explain(&scene.layers[2]);
        assert!(explanation.what.contains("GNOME"));
        assert!(explanation.what.contains("Mutter"));
    }

    #[test]
    fn every_example_is_in_the_catalog() {
        let catalog = catalog();
        let scenes = [full_desktop_scene(&catalog), compositor_scene(&catalog)];
        for layer in scenes.iter().flat_map(|scene| &scene.layers) {
            for example in explain(layer).examples {
                assert!(catalog.summary(example.kind, &example.id).is_some(), "{example:?} is not in the catalog");
            }
        }
    }

    #[test]
    fn scene_ids_round_trip() {
        for kind in [SceneKind::Yours, SceneKind::FullDesktop, SceneKind::CompositorOnly] {
            assert_eq!(SceneKind::from_id(kind.id()), Some(kind));
        }
    }
}
