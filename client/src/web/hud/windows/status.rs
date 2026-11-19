//! The entry of the list of windows for the Status window.

use super::*;

pub(in crate::web) static STATUS: Window = Window::usual("status", |app| app.hero.status_open, |_, _| framed(at(225, 285, 280, 141)), App::status_window, switched_off!(hero.status_open));
