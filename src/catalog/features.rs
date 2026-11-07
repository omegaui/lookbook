//! The few capabilities that decide what people can build on a compositor.

use gettextrs::gettext;

use super::Compositor;

/// A capability that Wayland protocols, or a desktop portal, can provide.
#[derive(Debug, Clone)]
pub struct KeyFeature {
    pub label: String,
    /// The feature is there when the compositor offers any of these globals.
    pub any_of: &'static [&'static str],
    /// Or when its portal backend provides this portal, for example "ScreenCast".
    pub portal: Option<&'static str>,
}

/// How a compositor provides a feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureSupport {
    Protocol,
    Portal(&'static str),
    Missing,
}

impl KeyFeature {
    pub fn support_on(&self, compositor: &Compositor) -> FeatureSupport {
        if self.any_of.iter().any(|global| compositor.supports(global)) {
            return FeatureSupport::Protocol;
        }

        match self.portal {
            Some(portal) if compositor.has_portal(portal) => FeatureSupport::Portal(portal),
            _ => FeatureSupport::Missing,
        }
    }
}

/// Bars, docks and launchers need layer shell; lock screens need session lock;
/// screenshot and recording tools need a capture protocol or the ScreenCast portal.
pub fn key_features() -> Vec<KeyFeature> {
    vec![
        KeyFeature {
            label: gettext("Layer shell"),
            any_of: &["zwlr_layer_shell_v1"],
            portal: None,
        },
        KeyFeature {
            label: gettext("Session lock"),
            any_of: &["ext_session_lock_manager_v1"],
            portal: None,
        },
        KeyFeature {
            label: gettext("Screen capture"),
            any_of: &["zwlr_screencopy_manager_v1", "ext_image_copy_capture_manager_v1"],
            portal: Some("ScreenCast"),
        },
    ]
}
