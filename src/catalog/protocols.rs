//! Plain-language descriptions of the Wayland protocols that matter to builders.

use gettextrs::gettext;
use serde::Deserialize;

use super::Catalog;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProtocolCategory {
    Shell,
    Capture,
    Windows,
    Security,
    Input,
    Outputs,
    Core,
}

impl ProtocolCategory {
    /// Every category, in the order the app shows them.
    pub const ALL: [Self; 7] = [
        Self::Shell,
        Self::Capture,
        Self::Windows,
        Self::Security,
        Self::Input,
        Self::Outputs,
        Self::Core,
    ];

    pub fn label(self) -> String {
        match self {
            Self::Shell => gettext("Shell"),
            Self::Capture => gettext("Screen capture"),
            Self::Windows => gettext("Windows and clipboard"),
            Self::Security => gettext("Session and security"),
            Self::Input => gettext("Input"),
            Self::Outputs => gettext("Outputs"),
            Self::Core => gettext("Core and graphics"),
        }
    }

    pub fn icon_name(self) -> &'static str {
        match self {
            Self::Shell => "lay-idea-bar-symbolic",
            Self::Capture => "camera-photo-symbolic",
            Self::Windows => "view-dual-symbolic",
            Self::Security => "system-lock-screen-symbolic",
            Self::Input => "input-keyboard-symbolic",
            Self::Outputs => "video-display-symbolic",
            Self::Core => "applications-graphics-symbolic",
        }
    }

    /// Category colors match the idea tiles: blue shell, orange capture, and so on.
    pub fn color(self) -> &'static str {
        match self {
            Self::Shell => "#1c71d8",
            Self::Capture => "#c64600",
            Self::Windows => "#26774d",
            Self::Security => "#813d9c",
            Self::Input => "#2a7f8a",
            Self::Outputs => "#9c6e03",
            Self::Core => "#5e5c64",
        }
    }
}

/// What one protocol lets apps do, for example layer shell.
#[derive(Debug, Clone, Deserialize)]
pub struct ProtocolInfo {
    pub interface: String,
    pub name: String,
    pub description: String,
    pub useful_for: String,
    pub category: ProtocolCategory,
}

/// A list of globals split into described protocols and the rest.
#[derive(Debug, Clone, Default)]
pub struct ProtocolOverview<'a> {
    /// In the catalog's order, which follows the categories.
    pub described: Vec<&'a ProtocolInfo>,
    pub others: Vec<String>,
}

impl Catalog {
    pub fn protocol_overview<'a>(&'a self, globals: &[String]) -> ProtocolOverview<'a> {
        let described = self
            .protocols
            .iter()
            .filter(|info| globals.contains(&info.interface))
            .collect();
        let others = globals
            .iter()
            .filter(|global| !self.protocols.iter().any(|info| &info.interface == *global))
            .cloned()
            .collect();

        ProtocolOverview { described, others }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundled_catalog() -> Catalog {
        let json = include_bytes!("../../data/resources/catalog/catalog.json");
        Catalog::from_json(json).expect("the bundled catalog should parse")
    }

    #[test]
    fn splits_described_protocols_from_the_rest() {
        let catalog = bundled_catalog();
        let globals = vec!["wl_compositor".to_owned(), "zwlr_layer_shell_v1".to_owned()];
        let overview = catalog.protocol_overview(&globals);

        assert_eq!(overview.described.len(), 1);
        assert_eq!(overview.described[0].name, "Layer shell");
        assert_eq!(overview.others, ["wl_compositor"]);
    }

    #[test]
    fn every_described_protocol_is_offered_somewhere() {
        let catalog = bundled_catalog();
        for info in &catalog.protocols {
            let offered = catalog.compositors.iter().any(|compositor| compositor.supports(&info.interface));
            assert!(offered, "no compositor in the catalog offers {}", info.interface);
        }
    }
}
