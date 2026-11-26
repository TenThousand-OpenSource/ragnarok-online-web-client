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

/// The same for a thing whose press says where on it the pointer was: what goes with the press is then that
/// place, counted from the thing's left and top in its own measure (`app/wiring.rs` writes it as `x,y`), in
/// the place of what the press was drawn with.
pub(super) fn pressed_at(press: impl Into<Press>) -> String {
    format!("{} data-place", pressed(press))
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

/// What marks a list or a text as one the wheel moves by steps, as the client's are moved, and not by pixels
/// as a page moves its own: each step so many pixels, and so many steps a notch (`crate::scrolling`; the
/// page's wheel is read in `app/wiring.rs`).
pub(super) fn stepped(step: i32, a_notch: i32) -> String {
    format!(r#"data-step="{step}" data-notch="{a_notch}""#)
}
