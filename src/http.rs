//! HTTP for everything the app reads live: one libsoup session with a disk
//! cache, and GET helpers.

use std::cell::RefCell;

use gtk::glib;
use soup::prelude::*;

use crate::config::VERSION;

const TIMEOUT_SECONDS: u32 = 30;
/// Screenshots, feeds and API answers kept between launches. Servers say how
/// long each stays fresh; unchanged ones are checked with a cheap request.
const DISK_CACHE_BYTES: u32 = 200 * 1024 * 1024;

thread_local! {
    static SHARED_SESSION: soup::Session = cached_session();
    static DISK_CACHE: RefCell<Option<soup::Cache>> = const { RefCell::new(None) };
}

/// One session for every read in the app, so connections and the disk cache
/// are shared.
pub fn shared_session() -> soup::Session {
    SHARED_SESSION.with(Clone::clone)
}

/// Writes the disk cache's index, so the next launch finds what is cached.
pub fn save_disk_cache() {
    DISK_CACHE.with(|cache| {
        if let Some(cache) = cache.borrow().as_ref() {
            cache.dump();
        }
    });
}

/// Names the app and its version and links to its repository, so source
/// owners know who is asking.
fn user_agent() -> String {
    format!("Syspedia/{VERSION} (+https://github.com/omegaui/syspedia)")
}

fn cached_session() -> soup::Session {
    let session = soup::Session::new();
    session.set_user_agent(&user_agent());
    session.set_timeout(TIMEOUT_SECONDS);

    let folder = glib::user_cache_dir().join("syspedia").join("http");
    let cache = soup::Cache::new(folder.to_str(), soup::CacheType::SingleUser);
    cache.set_max_size(DISK_CACHE_BYTES);
    cache.load();
    session.add_feature(&cache);
    DISK_CACHE.with(|slot| slot.replace(Some(cache)));
    session
}

pub async fn get_bytes(session: &soup::Session, url: &str) -> anyhow::Result<glib::Bytes> {
    let message = soup::Message::new("GET", url)?;
    send(session, &message, url).await
}

/// Like `get_bytes`, over HTTP/1.1. Some servers, Repology among them, reset
/// libsoup's HTTP/2 connections during the TLS handshake.
pub async fn get_bytes_over_http1(session: &soup::Session, url: &str) -> anyhow::Result<glib::Bytes> {
    let message = soup::Message::new("GET", url)?;
    message.set_force_http1(true);
    send(session, &message, url).await
}

async fn send(session: &soup::Session, message: &soup::Message, url: &str) -> anyhow::Result<glib::Bytes> {
    let bytes = session
        .send_and_read_future(message, glib::Priority::DEFAULT)
        .await?;

    let status = message.status();
    if status != soup::Status::Ok {
        anyhow::bail!("{url} answered {status:?}");
    }

    Ok(bytes)
}
