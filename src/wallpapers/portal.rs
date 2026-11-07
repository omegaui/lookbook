//! Sets the wallpaper through the XDG Wallpaper portal, which works from inside Flatpak.
//! GNOME and KDE provide it; many tiling compositors' portal backends do not.

use std::path::{Path, PathBuf};

use gtk::prelude::*;
use gtk::{gio, glib};

const PORTAL_BUS: &str = "org.freedesktop.portal.Desktop";
const PORTAL_PATH: &str = "/org/freedesktop/portal/desktop";
const WALLPAPER_INTERFACE: &str = "org.freedesktop.portal.Wallpaper";

/// Saves the image in the app's own cache, then asks the portal to use it as the background.
pub async fn set_wallpaper(image: &glib::Bytes, file_name: &str) -> anyhow::Result<()> {
    let path = save_to_cache(image, file_name)?;
    call_set_wallpaper_file(&path).await
}

/// The portal reads the image through a file descriptor, so it never needs our files.
async fn call_set_wallpaper_file(path: &Path) -> anyhow::Result<()> {
    let file = std::fs::File::open(path)?;
    let fd_list = gio::UnixFDList::new();
    let fd_index = fd_list.append(&file)?;

    let options = glib::VariantDict::new(None);
    options.insert("show-preview", false);
    options.insert("set-on", "background");

    // Built child by child, so the options stay a{sv} instead of being wrapped in a variant.
    let parameters = glib::Variant::tuple_from_iter([
        "".to_variant(),
        glib::variant::Handle(fd_index).to_variant(),
        options.end(),
    ]);

    let connection = gio::bus_get_future(gio::BusType::Session).await?;
    connection
        .call_with_unix_fd_list_future(
            Some(PORTAL_BUS),
            PORTAL_PATH,
            WALLPAPER_INTERFACE,
            "SetWallpaperFile",
            Some(&parameters),
            None,
            gio::DBusCallFlags::NONE,
            -1,
            Some(&fd_list),
        )
        .await?;

    Ok(())
}

/// Inside Flatpak the cache folder is the app's own, under ~/.var/app.
fn save_to_cache(image: &glib::Bytes, file_name: &str) -> anyhow::Result<PathBuf> {
    let folder = glib::user_cache_dir().join("wallpapers");
    std::fs::create_dir_all(&folder)?;

    let path = folder.join(file_name);
    std::fs::write(&path, image.as_ref())?;
    Ok(path)
}

/// The last part of an image address, without a query, for example "sea.png".
pub fn file_name_from_url(url: &str) -> String {
    let without_query = url.split('?').next().unwrap_or(url);
    let name = without_query.rsplit('/').next().unwrap_or(without_query);

    if name.is_empty() {
        return "wallpaper".to_owned();
    }
    name.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_names_drop_the_query() {
        assert_eq!(file_name_from_url("https://e/walls/sea.png?k=qBA7"), "sea.png");
        assert_eq!(file_name_from_url("https://e/uploads/2021/04/kswirl.jpg"), "kswirl.jpg");
        assert_eq!(file_name_from_url("https://e/"), "wallpaper");
    }
}
