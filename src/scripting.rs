//! Actions that let scripts drive the app without the mouse: scroll a page,
//! turn the spotlight carousel, open dialogs and a wallpaper's preview. They
//! are for checking the app and taking screenshots, so they exist only in
//! development builds, or when the app starts with SYSPEDIA_SCRIPTING=1:
//!
//!   flatpak run --env=SYSPEDIA_SCRIPTING=1 io.github.omegaui.Syspedia

use crate::config::PROFILE;

pub fn enabled() -> bool {
    PROFILE == "Devel" || std::env::var("SYSPEDIA_SCRIPTING").is_ok_and(|value| value == "1")
}
