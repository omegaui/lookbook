//! Sets the wallpaper through the XDG Wallpaper portal, which works from inside Flatpak.
//! GNOME and KDE provide it; many tiling compositors' portal backends do not.

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use gtk::prelude::*;
use gtk::{gio, glib};

const PORTAL_BUS: &str = "org.freedesktop.portal.Desktop";
const PORTAL_PATH: &str = "/org/freedesktop/portal/desktop";
const WALLPAPER_INTERFACE: &str = "org.freedesktop.portal.Wallpaper";
const REQUEST_INTERFACE: &str = "org.freedesktop.portal.Request";

/// How the portal answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallpaperOutcome {
    Set,
    /// The user said no, or closed the portal's dialog.
    Cancelled,
}

/// Saves the image in the app's own cache, then asks the portal to use it as the background.
pub async fn set_wallpaper(image: &glib::Bytes, file_name: &str) -> anyhow::Result<WallpaperOutcome> {
    let path = save_to_cache(image, file_name)?;
    call_set_wallpaper_file(&path).await
}

/// The portal reads the image through a file descriptor, so it never needs our
/// files. The call only starts a request; the answer comes later as the
/// request's Response signal, so the app listens for it before calling.
async fn call_set_wallpaper_file(path: &Path) -> anyhow::Result<WallpaperOutcome> {
    let file = std::fs::File::open(path)?;
    let fd_list = gio::UnixFDList::new();
    let fd_index = fd_list.append(&file)?;

    let connection = gio::bus_get_future(gio::BusType::Session).await?;
    let token = format!("syspedia{}", glib::random_int());
    let (answer, _subscription) = listen_for_response(&connection, &request_path(&connection, &token)?);

    let options = glib::VariantDict::new(None);
    options.insert("handle_token", &token);
    options.insert("show-preview", false);
    options.insert("set-on", "background");

    // Built child by child, so the options stay a{sv} instead of being wrapped in a variant.
    let parameters = glib::Variant::tuple_from_iter([
        "".to_variant(),
        glib::variant::Handle(fd_index).to_variant(),
        options.end(),
    ]);

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

    let code = answer.await.map_err(|_| anyhow::anyhow!("the wallpaper portal never answered"))?;
    outcome_from_response(code)
}

/// Where the portal puts the request: its path is built from our bus name and
/// the token, so we can listen before the call returns.
fn request_path(connection: &gio::DBusConnection, token: &str) -> anyhow::Result<String> {
    let unique_name = connection
        .unique_name()
        .ok_or_else(|| anyhow::anyhow!("the session bus gave no name"))?;
    let sender = unique_name.trim_start_matches(':').replace('.', "_");
    Ok(format!("{PORTAL_PATH}/request/{sender}/{token}"))
}

/// The response code arrives once; keep the subscription alive until then.
fn listen_for_response(
    connection: &gio::DBusConnection,
    request_path: &str,
) -> (futures_channel::oneshot::Receiver<u32>, gio::SignalSubscription) {
    let (sender, receiver) = futures_channel::oneshot::channel();
    let sender = RefCell::new(Some(sender));
    let subscription = connection.subscribe_to_signal(
        Some(PORTAL_BUS),
        Some(REQUEST_INTERFACE),
        Some("Response"),
        Some(request_path),
        None,
        gio::DBusSignalFlags::NONE,
        move |signal| {
            let code = signal.parameters.child_value(0).get::<u32>().unwrap_or(2);
            if let Some(sender) = sender.borrow_mut().take() {
                let _ = sender.send(code);
            }
        },
    );
    (receiver, subscription)
}

/// 0 means done, 1 that the user cancelled, anything else that it failed.
fn outcome_from_response(code: u32) -> anyhow::Result<WallpaperOutcome> {
    match code {
        0 => Ok(WallpaperOutcome::Set),
        1 => Ok(WallpaperOutcome::Cancelled),
        other => anyhow::bail!("the wallpaper portal answered {other}"),
    }
}

/// Inside Flatpak the cache folder is the app's own, under ~/.var/app.
fn save_to_cache(image: &glib::Bytes, file_name: &str) -> anyhow::Result<PathBuf> {
    let folder = glib::user_cache_dir().join("wallpapers");
    std::fs::create_dir_all(&folder)?;

    remove_older_wallpapers(&folder, file_name);
    let path = folder.join(file_name);
    std::fs::write(&path, image.as_ref())?;
    Ok(path)
}

/// The desktop keeps its own copy of the background, so the cache needs
/// only the one being set; without this the folder only grows.
fn remove_older_wallpapers(folder: &Path, keep: &str) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        if entry.file_name() != keep {
            let _ = std::fs::remove_file(entry.path());
        }
    }
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
    fn response_codes_mean_set_cancelled_or_failed() {
        assert_eq!(outcome_from_response(0).unwrap(), WallpaperOutcome::Set);
        assert_eq!(outcome_from_response(1).unwrap(), WallpaperOutcome::Cancelled);
        assert!(outcome_from_response(2).is_err());
    }

    #[test]
    fn file_names_drop_the_query() {
        assert_eq!(file_name_from_url("https://e/walls/sea.png?k=qBA7"), "sea.png");
        assert_eq!(file_name_from_url("https://e/uploads/2021/04/kswirl.jpg"), "kswirl.jpg");
        assert_eq!(file_name_from_url("https://e/"), "wallpaper");
    }
}
