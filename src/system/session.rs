use std::env;

/// The desktop session, read from environment variables that Flatpak passes through.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Session {
    /// The entries of `XDG_CURRENT_DESKTOP`, for example `["GNOME"]` or `["Hyprland"]`.
    pub desktop_names: Vec<String>,
    /// `XDG_SESSION_TYPE`, usually "wayland".
    pub session_type: String,
}

impl Session {
    pub fn detect() -> Self {
        Self {
            desktop_names: split_desktop_names(&env::var("XDG_CURRENT_DESKTOP").unwrap_or_default()),
            session_type: env::var("XDG_SESSION_TYPE").unwrap_or_default(),
        }
    }
}

/// `XDG_CURRENT_DESKTOP` is a colon-separated list, for example "Budgie:GNOME".
fn split_desktop_names(value: &str) -> Vec<String> {
    value
        .split(':')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_colon_separated_names() {
        assert_eq!(split_desktop_names("Budgie:GNOME"), ["Budgie", "GNOME"]);
        assert_eq!(split_desktop_names(""), Vec::<String>::new());
    }
}
