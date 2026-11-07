use gtk::glib;
use soup::prelude::*;

/// Names the app and links to its repository, so source owners know who is asking.
const USER_AGENT: &str = "LinuxAndYou/0.1 (+https://github.com/omegaui/linux_and_you)";
const TIMEOUT_SECONDS: u32 = 30;

pub fn new_session() -> soup::Session {
    let session = soup::Session::new();
    session.set_user_agent(USER_AGENT);
    session.set_timeout(TIMEOUT_SECONDS);
    session
}

pub async fn get_bytes(session: &soup::Session, url: &str) -> anyhow::Result<glib::Bytes> {
    let message = soup::Message::new("GET", url)?;
    let bytes = session
        .send_and_read_future(&message, glib::Priority::DEFAULT)
        .await?;

    let status = message.status();
    if status != soup::Status::Ok {
        anyhow::bail!("{url} answered {status:?}");
    }

    Ok(bytes)
}
