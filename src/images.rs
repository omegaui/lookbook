//! Pictures loaded live. Each address is decoded once per size and shared by
//! every widget that shows it, for as long as one of them does.

use std::cell::RefCell;
use std::collections::HashMap;

use gtk::prelude::*;
use gtk::{gdk, gdk_pixbuf, glib};

/// An address and the widest it was decoded at; 0 means full size.
type TextureKey = (String, i32);

thread_local! {
    static TEXTURES: RefCell<HashMap<TextureKey, glib::WeakRef<gdk::Texture>>> = RefCell::default();
}

/// The picture at full size.
pub async fn fetch_texture(session: &soup::Session, url: &str) -> anyhow::Result<gdk::Texture> {
    fetch(session, url, 0).await
}

/// The picture decoded no wider than `max_width` pixels, so a 4K README
/// image shown in a small card takes a fraction of the memory.
pub async fn fetch_texture_scaled(session: &soup::Session, url: &str, max_width: i32) -> anyhow::Result<gdk::Texture> {
    fetch(session, url, max_width).await
}

async fn fetch(session: &soup::Session, url: &str, max_width: i32) -> anyhow::Result<gdk::Texture> {
    let key = (url.to_owned(), max_width);
    if let Some(texture) = cached(&key) {
        return Ok(texture);
    }
    let bytes = crate::http::get_bytes(session, url).await?;
    let texture = if max_width > 0 {
        decode_scaled(&bytes, max_width)?
    } else {
        gdk::Texture::from_bytes(&bytes)?
    };
    remember(key, &texture);
    Ok(texture)
}

/// Asks the decoder for a smaller size before it decodes, so the full image
/// never sits in memory.
fn decode_scaled(bytes: &glib::Bytes, max_width: i32) -> anyhow::Result<gdk::Texture> {
    let loader = gdk_pixbuf::PixbufLoader::new();
    loader.connect_size_prepared(move |loader, width, height| {
        if let Some((width, height)) = scaled_size(width, height, max_width) {
            loader.set_size(width, height);
        }
    });
    loader.write_bytes(bytes)?;
    loader.close()?;
    let pixbuf = loader
        .pixbuf()
        .ok_or_else(|| anyhow::anyhow!("the picture did not decode"))?;
    Ok(gdk::Texture::for_pixbuf(&pixbuf))
}

/// The size to decode at, or None when the picture is already narrow enough.
fn scaled_size(width: i32, height: i32, max_width: i32) -> Option<(i32, i32)> {
    if width <= max_width || width <= 0 {
        return None;
    }
    let scaled_height = i64::from(height) * i64::from(max_width) / i64::from(width);
    Some((max_width, (scaled_height as i32).max(1)))
}

fn cached(key: &TextureKey) -> Option<gdk::Texture> {
    TEXTURES.with(|textures| textures.borrow().get(key).and_then(glib::WeakRef::upgrade))
}

/// Keeps a weak reference: the picture is freed when no widget shows it.
fn remember(key: TextureKey, texture: &gdk::Texture) {
    TEXTURES.with(|textures| {
        let mut textures = textures.borrow_mut();
        textures.retain(|_, weak| weak.upgrade().is_some());
        textures.insert(key, texture.downgrade());
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_pictures_shrink_and_keep_their_shape() {
        assert_eq!(scaled_size(3840, 2160, 960), Some((960, 540)));
        assert_eq!(scaled_size(800, 600, 960), None);
    }
}
