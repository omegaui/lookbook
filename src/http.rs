//! HTTP for everything the app reads live: one libsoup session setup and a GET helper.

use gtk::glib;
use soup::prelude::*;

/// Names the app and links to its repository, so source owners know who is asking.
const USER_AGENT: &str = "LinuxAndYou/0.1 (+https://github.com/omegaui/linux_and_you)";
const TIMEOUT_SECONDS: u32 = 30;

thread_local! {
    static SHARED_SESSION: soup::Session = new_session();
}

/// One session for small reads all over the app, so connections are reused.
pub fn shared_session() -> soup::Session {
    SHARED_SESSION.with(Clone::clone)
}

pub fn new_session() -> soup::Session {
    let session = soup::Session::new();
    session.set_user_agent(USER_AGENT);
    session.set_timeout(TIMEOUT_SECONDS);
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
