//! roweb: a Ragnarok Online client for the browser.

pub mod bmp;
pub mod i18n;
pub mod map;
pub mod model;
pub mod net;
pub mod sprite;

#[cfg(target_arch = "wasm32")]
mod web;
