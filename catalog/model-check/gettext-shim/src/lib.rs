//! Returns the text unchanged, like gettext with no translations.

pub fn gettext<T: Into<String>>(msgid: T) -> String {
    msgid.into()
}
