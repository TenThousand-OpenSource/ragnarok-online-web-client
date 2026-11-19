//! The pieces every window on the page is made of, whichever screen it belongs to.

use crate::press::Press;
use crate::press::things::{self, Drag, Drop, Form, Menu, Use};

/// Where the client keeps the pictures of its windows.
pub(super) const UI: &str = "data/texture/유저인터페이스/";

/// The address of one of the client's pictures of its windows, as the gateway serves it.
pub(super) fn picture(name: &str) -> String {
    super::image_url(&format!("{UI}{name}"))
}

/// A place and a size, as a style.
pub(super) fn at(x: i32, y: i32, width: i32, height: i32) -> String {
    format!("left:{x}px;top:{y}px;width:{width}px;height:{height}px")
}

/// What marks a thing on the page as pressable, for its tag: the press's name and what goes with it. The app
/// reads the press back from these when the thing is pressed (`crate::press`).
pub(super) fn pressed(press: impl Into<Press>) -> String {
    let (what, arg) = press.into().name();
    format!(r#"data-click="{what}" data-arg="{}""#, super::app::escape(&arg))
}

/// What marks a thing as one a double click does something with (`crate::press::things`).
pub(super) fn used(thing: Use) -> String {
    format!(r#"data-use="{}""#, things::written(&thing))
}

/// What marks a thing as one that can be dragged out of its place.
pub(super) fn dragged(thing: Drag) -> String {
    format!(r#"data-drag="{}""#, things::written(&thing))
}

/// What marks a window, or a part of one, as taking the things dragged that are let go over it.
pub(super) fn dropped(onto: Drop) -> String {
    format!(r#"data-drop="{}""#, things::written(&onto))
}

/// What marks a thing as one the right button opens a menu on.
pub(super) fn menued(thing: Menu) -> String {
    format!(r#"data-menu="{}""#, things::written(&thing))
}

/// What marks a form as the one it is, for when it is sent.
pub(super) fn sent(form: Form) -> String {
    format!(r#"data-submit="{}""#, things::written(&form))
}
