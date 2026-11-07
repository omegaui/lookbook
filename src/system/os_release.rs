use std::fs;

/// Inside Flatpak, the host's file is mounted at `/run/host/os-release`.
/// `/etc/os-release` there describes the Flatpak runtime, not the user's distro.
const OS_RELEASE_PATHS: [&str; 3] = ["/run/host/os-release", "/etc/os-release", "/usr/lib/os-release"];

/// The fields of the os-release file that Linux & You uses.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OsRelease {
    pub id: String,
    pub name: String,
    pub pretty_name: String,
    pub variant_id: Option<String>,
    pub logo: Option<String>,
}

impl OsRelease {
    pub fn load() -> Self {
        let text = read_first_available_file();
        Self::parse(&text)
    }

    pub fn parse(text: &str) -> Self {
        let mut release = Self::default();

        for line in text.lines() {
            let Some((key, value)) = split_entry(line) else {
                continue;
            };

            match key {
                "ID" => release.id = value,
                "NAME" => release.name = value,
                "PRETTY_NAME" => release.pretty_name = value,
                "VARIANT_ID" => release.variant_id = Some(value),
                "LOGO" => release.logo = Some(value),
                _ => {}
            }
        }

        release
    }
}

fn read_first_available_file() -> String {
    OS_RELEASE_PATHS
        .iter()
        .find_map(|path| fs::read_to_string(path).ok())
        .unwrap_or_default()
}

/// Splits `KEY="value"` into its key and unquoted value. Skips comments and blank lines.
fn split_entry(line: &str) -> Option<(&str, String)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }

    let (key, value) = line.split_once('=')?;
    let value = value.trim().trim_matches(|c| c == '"' || c == '\'');

    Some((key.trim(), value.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_quoted_and_plain_values() {
        let text = "NAME=\"Fedora Linux\"\nID=fedora\nVARIANT_ID=workstation\nLOGO=fedora-logo-icon\n";
        let release = OsRelease::parse(text);

        assert_eq!(release.name, "Fedora Linux");
        assert_eq!(release.id, "fedora");
        assert_eq!(release.variant_id.as_deref(), Some("workstation"));
        assert_eq!(release.logo.as_deref(), Some("fedora-logo-icon"));
    }

    #[test]
    fn skips_comments_and_blank_lines() {
        let text = "# A comment = with an equals sign\n\nID='arch'\n";
        let release = OsRelease::parse(text);

        assert_eq!(release.id, "arch");
        assert_eq!(release.variant_id, None);
    }
}
