//! The navigation's tables (`crate::navigation::Atlas`), as the gateway gives them out of the client's
//! `navi_*.lub` (`gateway/src/tables/navigation.rs`). They are large and are asked for by themselves, as the
//! game is entered, not with the other tables.

use std::cell::Cell;
use std::rc::Rc;

use super::*;
use crate::navigation::atlas::{LinkRow, Rows, SpotRow, WayRow};
use crate::navigation::Atlas;

thread_local! {
    static ATLAS: RefCell<Option<Rc<Atlas>>> = const { RefCell::new(None) };
    static ASKED: Cell<bool> = const { Cell::new(false) };
}

/// What the navigation knows of the world; none until the tables are read (`want_atlas`).
pub fn atlas() -> Option<Rc<Atlas>> {
    ATLAS.with_borrow(Clone::clone)
}

/// Asks the gateway for the navigation's tables, once, and calls `then` when they are read.
pub fn want_atlas(then: fn()) {
    if ASKED.replace(true) {
        return;
    }
    wasm_bindgen_futures::spawn_local(async move {
        match fetch_json("/tables/navigation.json").await {
            Ok(tables) => {
                ATLAS.set(Some(Rc::new(Atlas::new(rows(&tables)))));
                then();
            }
            Err(error) => {
                // Asked for again the next time: a gateway that was not there yet may be by then.
                ASKED.set(false);
                web_sys::console::warn_1(&format!("navigation: {error}").into());
            }
        }
    });
}

fn whole(row: &js_sys::Array, at: u32) -> i32 {
    row.get(at).as_f64().unwrap_or(0.0) as i32
}

fn words(row: &js_sys::Array, at: u32) -> String {
    row.get(at).as_string().unwrap_or_default()
}

fn spot(entry: &wasm_bindgen::JsValue) -> SpotRow {
    let row = row(entry);
    (words(&row, 0), whole(&row, 1), whole(&row, 2), whole(&row, 3), words(&row, 4), words(&row, 5), whole(&row, 6), whole(&row, 7))
}

fn link(entry: &wasm_bindgen::JsValue) -> LinkRow {
    let (map, id, kind, index, name, title, x, y) = spot(entry);
    let row = row(entry);
    (map, id, kind, index, name, title, x, y, words(&row, 8), whole(&row, 9), whole(&row, 10))
}

fn way(entry: &wasm_bindgen::JsValue) -> WayRow {
    let entry = row(entry);
    let passes = list(&entry.get(2)).iter().map(|pass| row(pass)).map(|pass| (words(&pass, 0), whole(&pass, 1), whole(&pass, 2))).collect();
    (words(&entry, 0), whole(&entry, 1), passes)
}

/// The gateway's answer as the rows the atlas is made of.
fn rows(tables: &wasm_bindgen::JsValue) -> Rows {
    let of = |name: &str| list(&field(tables, name));
    Rows {
        maps: of("maps").iter().map(|entry| row(entry)).map(|row| (words(&row, 0), words(&row, 1), whole(&row, 2), whole(&row, 3), whole(&row, 4))).collect(),
        npcs: of("npcs").iter().map(spot).collect(),
        mobs: of("mobs").iter().map(spot).collect(),
        links: of("links").iter().map(link).collect(),
        ways: of("ways").iter().map(way).collect(),
        npc_ways: of("npc_ways").iter().map(way).collect(),
        picks: of("picks")
            .iter()
            .map(|entry| row(entry))
            .map(|row| (words(&row, 0), whole(&row, 1), whole(&row, 2), std::array::from_fn(|place| whole(&row, place as u32 + 3))))
            .collect(),
    }
}
