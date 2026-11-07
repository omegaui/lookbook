//! The few protocols that decide what people can build on a compositor.

use gettextrs::gettext;

use super::Compositor;

/// A capability that one of several Wayland protocols provides.
#[derive(Debug, Clone)]
pub struct KeyFeature {
    pub label: String,
    /// The feature is there when the compositor offers any of these globals.
    pub any_of: &'static [&'static str],
}

impl KeyFeature {
    pub fn is_supported_by(&self, compositor: &Compositor) -> bool {
        self.any_of.iter().any(|global| compositor.supports(global))
    }
}

/// Bars, docks and launchers need layer shell; lock screens need session lock;
/// screenshot and recording tools need one of the capture protocols.
pub fn key_features() -> Vec<KeyFeature> {
    vec![
        KeyFeature {
            label: gettext("Layer shell"),
            any_of: &["zwlr_layer_shell_v1"],
        },
        KeyFeature {
            label: gettext("Session lock"),
            any_of: &["ext_session_lock_manager_v1"],
        },
        KeyFeature {
            label: gettext("Screen capture"),
            any_of: &["zwlr_screencopy_manager_v1", "ext_image_copy_capture_manager_v1"],
        },
    ]
}
