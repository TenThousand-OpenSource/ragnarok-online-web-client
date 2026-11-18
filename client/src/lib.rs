//! roweb: a Ragnarok Online client for the browser.

pub mod blow;
pub mod bmp;
pub mod cursor;
pub mod effect;
pub mod effect_tables;
pub mod hotkeys;
pub mod i18n;
pub mod jobs;
pub mod map;
pub mod model;
pub mod mounts;
pub mod net;
pub mod press;
pub mod prim;
pub mod sprite;
pub mod stack;
pub mod talk;

#[cfg(target_arch = "wasm32")]
mod web;
