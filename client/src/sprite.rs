//! The client's sprites: SPR (the pictures), ACT (how they are laid out per action and frame) and PAL (a recolour).
//! Drawing a frame of them into an RGBA picture is in `sprite/canvas.rs`; a frame as the client's effects draw it,
//! a quad to a layer, in `sprite/layers.rs`.
//! `spr.rs` reads the pictures and the palettes, `act.rs` the actions; `imf.rs` is the order a character's
//! parts are drawn in.
//!
//! Format facts follow the layouts documented by korangar's `ragnarok-formats` (MIT) and rustnarok-client's
//! `src/sprite.rs` (MIT OR Apache-2.0).

mod act;
mod canvas;
pub mod imf;
mod layers;
mod spr;

pub use act::*;
pub use canvas::Canvas;
pub use layers::{LayerQuad, layer_quads};
pub use spr::*;

#[cfg(test)]
mod tests;
