use gtk::gio;

use super::Catalog;

const BUNDLED_CATALOG_PATH: &str = "/io/github/omegaui/LinuxAndYou/catalog/catalog.json";

impl Catalog {
    /// Loads the catalog snapshot that ships inside the app.
    pub fn load_bundled() -> anyhow::Result<Self> {
        let bytes =
            gio::resources_lookup_data(BUNDLED_CATALOG_PATH, gio::ResourceLookupFlags::NONE)?;
        Self::from_json(&bytes)
    }

    pub fn from_json(bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
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
    fn bundled_catalog_has_items() {
        let catalog = bundled_catalog();

        assert!(!catalog.distros.is_empty());
        assert!(!catalog.desktops.is_empty());
        assert!(!catalog.compositors.is_empty());
    }

    #[test]
    fn collection_members_exist() {
        let catalog = bundled_catalog();

        for collection in &catalog.collections {
            for member in &collection.members {
                assert!(
                    catalog.summary(collection.kind, member).is_some(),
                    "collection \"{}\" names \"{member}\", which is not in the catalog",
                    collection.title
                );
            }
        }
    }

    #[test]
    fn every_logo_is_bundled() {
        let catalog = bundled_catalog();
        let resource_list = include_str!("../../data/resources/resources.gresource.xml");

        for logo in catalog_logos(&catalog) {
            let file = format!("icons/scalable/apps/lay-logo-{logo}-symbolic.svg");
            let path = format!("{}/data/resources/{file}", env!("CARGO_MANIFEST_DIR"));

            assert!(std::path::Path::new(&path).exists(), "{path} does not exist");
            assert!(
                resource_list.contains(&file),
                "{file} is not listed in resources.gresource.xml"
            );
        }
    }

    fn catalog_logos(catalog: &Catalog) -> Vec<&str> {
        let distros = catalog.distros.iter().filter_map(|item| item.logo.as_deref());
        let desktops = catalog.desktops.iter().filter_map(|item| item.logo.as_deref());
        let compositors = catalog.compositors.iter().filter_map(|item| item.logo.as_deref());
        distros.chain(desktops).chain(compositors).collect()
    }

    #[test]
    fn every_compositor_has_protocol_data() {
        let catalog = bundled_catalog();

        for compositor in &catalog.compositors {
            assert!(
                compositor.supports("wl_compositor"),
                "{} has no protocol list",
                compositor.id
            );
        }
    }

    #[test]
    fn mutter_has_no_layer_shell_and_sway_does() {
        let catalog = bundled_catalog();
        let mutter = catalog.find_compositor("mutter").unwrap();
        let sway = catalog.find_compositor("sway").unwrap();

        assert!(!mutter.supports("zwlr_layer_shell_v1"));
        assert!(sway.supports("zwlr_layer_shell_v1"));
    }

    #[test]
    fn spotlights_point_to_items() {
        let catalog = bundled_catalog();

        for spotlight in &catalog.spotlights {
            assert!(
                catalog.summary(spotlight.kind, &spotlight.id).is_some(),
                "spotlight \"{}\" is not in the catalog",
                spotlight.id
            );
        }
    }
}
