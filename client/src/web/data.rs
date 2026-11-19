//! The game's own tables, as the gateway gives them out of the client's files.
//!
//! Each kind of table has its file here, with what it keeps, how it is read and how it is asked: `jobs`, `items`, `cards`,
//! `skills`, `pets`, `interface`, `maps`, `guide`, `statuses`, `commands`, `song_talk`. This file has what they share (fetching, reading JSON) and
//! `load`, which fetches the tables once and hands them to each. Everything is reached as `data::…`, whichever
//! file it is in.

use std::cell::RefCell;
use std::collections::HashMap;

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use super::js_error;

mod ai;
mod cards;
mod commands;
mod figures;
mod guide;
mod interface;
mod items;
mod jobs;
mod maps;
mod pets;
mod quests;
mod scenery;
mod skills;
mod song_talk;
mod statuses;
mod tips;
mod worldmap;

pub use ai::ai_scripts;
pub use cards::*;
pub use commands::*;
pub use figures::{head_layers, shadow};
pub use guide::*;
pub use interface::*;
pub use items::*;
pub use jobs::*;
pub use maps::*;
pub use pets::*;
pub use quests::*;
pub use scenery::{Sky, sky};
pub use skills::*;
pub use song_talk::song_line;
pub use statuses::*;
pub use tips::*;
pub use worldmap::*;

/// Whether the game speaks Russian now: the items' and skills' texts are then asked for in Russian.
fn russian() -> bool {
    crate::i18n::spoken() == crate::i18n::Lang::Ru
}

/// Forgets the items' and skills' texts: after the language changed, they are asked for anew in the new one.
pub fn forget_texts() {
    items::forget();
    skills::forget();
}

async fn fetch_json(url: &str) -> Result<wasm_bindgen::JsValue, String> {
    let window = web_sys::window().ok_or("no window")?;
    let response: web_sys::Response = JsFuture::from(window.fetch_with_str(url)).await.map_err(js_error)?.dyn_into().map_err(js_error)?;
    if !response.ok() {
        return Err(format!("{url}: HTTP {}", response.status()));
    }
    JsFuture::from(response.json().map_err(js_error)?).await.map_err(js_error)
}

/// A JSON object's entries whose keys are numbers.
fn entries<Id: std::str::FromStr>(object: &wasm_bindgen::JsValue) -> Vec<(Id, wasm_bindgen::JsValue)> {
    let Ok(object) = object.clone().dyn_into::<js_sys::Object>() else { return Vec::new() };
    js_sys::Object::entries(&object)
        .iter()
        .filter_map(|entry| {
            let entry: js_sys::Array = entry.dyn_into().ok()?;
            Some((entry.get(0).as_string()?.parse().ok()?, entry.get(1)))
        })
        .collect()
}

fn text_of(object: &wasm_bindgen::JsValue, field: &str) -> Option<String> {
    js_sys::Reflect::get(object, &field.into()).ok()?.as_string()
}

/// A field of a JSON object; undefined when it has none.
fn field(object: &wasm_bindgen::JsValue, name: &str) -> wasm_bindgen::JsValue {
    js_sys::Reflect::get(object, &name.into()).unwrap_or(wasm_bindgen::JsValue::UNDEFINED)
}

/// Fetches the tables from the gateway and hands them to each kind's file. Until they are there, jobs have their
/// built-in names and nobody wears headgear.
pub async fn load() -> Result<(), String> {
    // The number changes when the tables gain something, so a browser's older copy is not used in their place.
    let tables = fetch_json("/tables.json?v=26").await?;
    jobs::read(&tables);
    skills::read(&tables);
    pets::read(&tables);
    quests::read(&tables);
    interface::read(&tables);
    maps::read(&tables);
    guide::read(&tables);
    statuses::read(&tables);
    commands::read(&tables);
    worldmap::read(&tables);
    tips::read(&tables);
    figures::read(&tables);
    cards::read(&tables);
    scenery::read(&tables);
    // (Not of the gateway's tables: two files of the client's data, read as they are.)
    song_talk::fetch();
    skills::read_russian().await;
    Ok(())
}
