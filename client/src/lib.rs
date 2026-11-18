//! roweb: a Ragnarok Online client for the browser.

pub mod bmp;
pub mod cursor;
pub mod effect;
pub mod effect_tables;
pub mod hotkeys;
pub mod i18n;
pub mod map;
pub mod model;
pub mod net;
pub mod prim;
pub mod sprite;
pub mod talk;

#[cfg(target_arch = "wasm32")]
mod web;
