//! Lists the Wayland globals the compositor shows to Linux & You.
//!
//! Inside Flatpak, compositors hide privileged globals (layer shell, screen
//! capture, session lock and others) from the app. So this list is "what a
//! sandboxed app can see", not everything the compositor offers. The ideas use
//! the catalog's full list for the detected compositor instead.

use wayland_client::protocol::wl_registry;
use wayland_client::{Connection, Dispatch, QueueHandle};

/// Collects global interface names as the registry announces them.
#[derive(Debug, Default)]
struct GlobalList {
    names: Vec<String>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for GlobalList {
    fn event(
        state: &mut Self,
        _registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &(),
        _connection: &Connection,
        _queue: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { interface, .. } = event {
            state.names.push(interface);
        }
    }
}

/// Returns an empty list when there is no Wayland session.
pub fn visible_globals() -> Vec<String> {
    match list_globals() {
        Ok(names) => names,
        Err(error) => {
            tracing::warn!("Could not list Wayland globals: {error:#}");
            Vec::new()
        }
    }
}

fn list_globals() -> anyhow::Result<Vec<String>> {
    let connection = Connection::connect_to_env()?;
    let mut queue = connection.new_event_queue();
    let queue_handle = queue.handle();
    connection.display().get_registry(&queue_handle, ());

    let mut list = GlobalList::default();
    queue.roundtrip(&mut list)?;

    list.names.sort();
    list.names.dedup();
    Ok(list.names)
}
