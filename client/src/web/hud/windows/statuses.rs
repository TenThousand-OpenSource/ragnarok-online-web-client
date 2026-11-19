//! The entry of the list of windows for the states down the right side of the screen.

use super::*;

/// The states the character is in, down the right side of the screen: its pictures stand by the right edge, so
/// the element is a point at the corner; nothing of it but the pictures takes the mouse.
pub(in crate::web) static STATUSES: Window =
    Window { id: "statuses", open: ALWAYS, frame: |_, _| r#"class="a" style="right:0;top:0;width:0;height:0""#.to_owned(), draw: App::statuses_column, order: None, layer: Layer::Usual, drawn: None };
