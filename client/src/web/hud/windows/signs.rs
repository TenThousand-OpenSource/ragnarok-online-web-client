//! The entry of the list of windows for the signs over heads (a chat room's, a shop's): the place the world
//! keeps them in.

use super::*;

/// Where the signs over heads are kept (`world/signs.rs` writes and moves them, frame after frame): the whole
/// stage, under the windows, itself letting the pointer through.
pub(in crate::web) static SIGNS: Window = Window { id: "signs", open: ALWAYS, frame: |_, _| r#"class="a" style="left:0;top:0;width:0;height:0""#.to_owned(), draw: |_| String::new(), order: None, layer: Layer::Under, drawn: None };
