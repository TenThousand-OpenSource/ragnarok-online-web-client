//! The battlegrounds one can queue for (the gateway's `tables/battleground.rs`, the client's
//! `EntryQueueList.lub`), asked as the client's `g_entryQueueList` is: all of them in the order of their
//! numbers, and one by its number or by the name the server knows it by.

use super::*;
use crate::battleground::Entry;

thread_local! {
    static ENTRIES: RefCell<std::rc::Rc<Vec<Entry>>> = RefCell::new(std::rc::Rc::new(Vec::new()));
}

/// The battlegrounds, in the order of their numbers.
pub fn battlegrounds() -> std::rc::Rc<Vec<Entry>> {
    ENTRIES.with_borrow(Clone::clone)
}

/// The battleground of that number.
pub fn battleground(id: u32) -> Option<Entry> {
    ENTRIES.with_borrow(|entries| entries.iter().find(|entry| entry.id == id).cloned())
}

/// The battleground the server knows by that name.
pub fn battleground_named(name: &str) -> Option<Entry> {
    ENTRIES.with_borrow(|entries| entries.iter().find(|entry| entry.name == name).cloned())
}

/// Takes the battlegrounds out of what the gateway gave. An older gateway gives none.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    let whole = |entry: &wasm_bindgen::JsValue, name: &str| number(entry, name).unwrap_or(0.0) as u32;
    let words = |entry: &wasm_bindgen::JsValue, name: &str| text_of(entry, name).unwrap_or_default();
    let entries = list(&field(tables, "battleground"))
        .into_iter()
        .map(|entry| Entry {
            id: whole(&entry, "id"),
            name: words(&entry, "name"),
            shown: words(&entry, "shown"),
            allies: whole(&entry, "allies"),
            enemies: whole(&entry, "enemies"),
            level_kind: whole(&entry, "level_kind") as u8,
            level: whole(&entry, "level"),
            level_end: whole(&entry, "level_end"),
            win: words(&entry, "win"),
            draw: words(&entry, "draw"),
            loss: words(&entry, "loss"),
            goal: words(&entry, "goal"),
            picture: words(&entry, "picture"),
        })
        .collect();
    ENTRIES.set(std::rc::Rc::new(entries));
}
