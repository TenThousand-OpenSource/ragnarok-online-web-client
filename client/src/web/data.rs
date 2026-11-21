//! The game's own tables, as the gateway gives them out of the client's files.
//!
//! Each kind of table has its file here, with what it keeps, how it is read and how it is asked: `jobs`, `items`, `cards`,
//! `skills`, `pets`, `interface`, `maps`, `guide`, `statuses`, `commands`, `song_talk`. This file has what they share (fetching, reading JSON) and
//! `load`, which fetches the tables once and hands them to each. Everything is reached as `data::…`, whichever
//! file it is in.
//!
//! What the files share in walking the gateway's JSON: `entries` (an object keyed by numbers), `field`,
//! `text_of`, `number`, `flag` (a field of an object), `list`, `texts` (a list).

use std::cell::RefCell;
use std::collections::HashMap;

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use super::js_error;

mod ai;
mod aura;
mod books;
mod cards;
mod castles;
mod commands;
mod emitters;
mod figures;
mod guide;
mod interface;
mod items;
mod jobs;
mod manner;
mod maps;
mod monster_skills;
mod pets;
mod quests;
mod scenery;
mod skill_needs;
mod skills;
mod song_talk;
mod statuses;
mod tips;
mod worldmap;

pub use ai::ai_scripts;
pub use aura::aura_table;
pub use books::{book_text, is_book};
pub use cards::*;
pub use castles::castle;
pub use commands::*;
pub use emitters::emitters;
pub use figures::{head_layers, shadow};
pub use guide::*;
pub use interface::*;
pub use items::*;
pub use jobs::*;
pub use manner::{foul, starred};
pub use maps::*;
pub use monster_skills::monster_skill;
pub use pets::*;
pub use quests::*;
pub use scenery::{Sky, sky};
pub use skill_needs::{skill_asks, skill_sp};
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

/// A JSON list as its entries; none of what is no list, or is not there at all (a field an older gateway
/// does not give). It never fails: `Array.from` of the browser, which throws for `undefined` and `null`, is
/// not asked.
fn list(value: &wasm_bindgen::JsValue) -> Vec<wasm_bindgen::JsValue> {
    row(value).to_vec()
}

/// A JSON list as it is, to be asked for its places by their numbers; an empty one for what is no list.
fn row(value: &wasm_bindgen::JsValue) -> js_sys::Array {
    value.dyn_ref::<js_sys::Array>().cloned().unwrap_or_default()
}

/// A JSON object's entries whose keys are numbers (or what else `Id` is read from) and whose values are
/// texts, each key with its text; an entry of another kind is passed by.
fn names<Id: std::str::FromStr>(object: &wasm_bindgen::JsValue) -> Vec<(Id, String)> {
    entries::<Id>(object).into_iter().filter_map(|(key, name)| Some((key, name.as_string()?))).collect()
}

/// The texts of a JSON list of texts; what is no text in it is passed by, and what is no list has none, as
/// for `list`.
fn texts(value: &wasm_bindgen::JsValue) -> Vec<String> {
    list(value).iter().filter_map(wasm_bindgen::JsValue::as_string).collect()
}

/// A number in a field of a JSON object; none when the object has no such field, or it is no number.
fn number(object: &wasm_bindgen::JsValue, name: &str) -> Option<f64> {
    field(object, name).as_f64()
}

/// Whether a field of a JSON object is `true`; `false` also when the object has no such field.
fn flag(object: &wasm_bindgen::JsValue, name: &str) -> bool {
    field(object, name).as_bool().unwrap_or(false)
}

/// What a skill is called, in the player's language where there is a translation; `otherwise` gives what
/// stands for the name of one the tables do not know (each window has its own words for that).
pub fn skill_name(id: u16, otherwise: impl FnOnce() -> String) -> String {
    skill(id).map_or_else(otherwise, |(_, name, _)| name)
}

/// What a map is called on its sign (`map_sign`), by its name without `.gat`; the name itself for a map that
/// has no sign.
pub fn map_name(map: &str) -> String {
    map_sign(map).map_or_else(|| map.to_owned(), |sign| sign.name)
}

/// Fetches the tables from the gateway and hands them to each kind's file. Until they are there, jobs have their
/// built-in names and nobody wears headgear.
pub async fn load() -> Result<(), String> {
    // The number changes when the tables gain something, so a browser's older copy is not used in their place.
    let tables = fetch_json("/tables.json?v=31").await?;
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
    aura::read(&tables);
    manner::read(&tables);
    monster_skills::read(&tables);
    castles::read(&tables);
    skill_needs::read(&tables);
    // (Not of the gateway's tables: two files of the client's data, read as they are.)
    song_talk::fetch();
    books::fetch();
    skills::read_russian().await;
    Ok(())
}
