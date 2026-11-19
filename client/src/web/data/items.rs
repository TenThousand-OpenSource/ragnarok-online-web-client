//! The items: names, pictures and descriptions, asked for from the gateway as they are met.

use super::*;

thread_local! {
    /// Items by ID, as far as they were asked for: what they are called, and the name of their pictures.
    static ITEMS: RefCell<HashMap<u32, (String, String)>> = RefCell::new(HashMap::new());
    /// What more is told of an item looked at: the client's description, line by line, and what the server's
    /// database says of it.
    static ABOUT: RefCell<HashMap<u32, (Vec<String>, Vec<(String, String)>)>> = RefCell::new(HashMap::new());
    /// What an item is before it is identified: its name, the name of its pictures, its description.
    static UNIDENTIFIED: RefCell<HashMap<u32, (String, String, Vec<String>)>> = RefCell::new(HashMap::new());
    /// How many slots for cards an item has (the table's `slotCount`), for those that have any.
    static SLOTS: RefCell<HashMap<u32, u8>> = RefCell::new(HashMap::new());
    /// Items asked for and not answered yet (or not known to the gateway at all).
    static ITEMS_ASKED: RefCell<std::collections::HashSet<u32>> = RefCell::new(std::collections::HashSet::new());
}

/// What an item is called and the name of its pictures, once the gateway has told (see `want_items`).
pub fn item(id: u32) -> Option<(String, String)> {
    ITEMS.with_borrow(|items| items.get(&id).cloned())
}

/// What an item in someone's hands is called and the name of its pictures: its own, or, until it is identified,
/// what the client's table gives for it unidentified (as the client's `CSession::IconPath` and `DisplayName`).
pub fn item_as(id: u32, identified: bool) -> Option<(String, String)> {
    let known = item(id)?;
    if identified {
        return Some(known);
    }
    Some(UNIDENTIFIED.with_borrow(|unknown| unknown.get(&id).filter(|(name, ..)| !name.is_empty()).map(|(name, resource, _)| (name.clone(), resource.clone()))).unwrap_or(known))
}

/// How many slots for cards an item has: what the client writes as `[n]` after its name.
pub fn item_slots(id: u32) -> u8 {
    SLOTS.with_borrow(|slots| slots.get(&id).copied().unwrap_or(0))
}

/// The description of an item not identified yet.
pub fn unidentified_about(id: u32) -> Vec<String> {
    UNIDENTIFIED.with_borrow(|unknown| unknown.get(&id).map(|(_, _, lines)| lines.clone()).unwrap_or_default())
}

/// An item's description and facts, once the gateway has told of the item.
pub fn about(id: u32) -> Option<(Vec<String>, Vec<(String, String)>)> {
    ABOUT.with_borrow(|about| about.get(&id).cloned())
}

/// Asks the gateway about the items not known yet, and calls `then` when it has answered.
pub fn want_items(ids: impl Iterator<Item = u32>, then: fn()) {
    let missing: Vec<u32> = ids.filter(|id| *id != 0 && !ITEMS.with_borrow(|items| items.contains_key(id)) && ITEMS_ASKED.with_borrow_mut(|asked| asked.insert(*id))).collect();
    if missing.is_empty() {
        return;
    }
    wasm_bindgen_futures::spawn_local(async move {
        let list: Vec<String> = missing.iter().map(u32::to_string).collect();
        match fetch_json(&format!("/tables/{}items/{}", if russian() { "ru/" } else { "" }, list.join(","))).await {
            Ok(answer) => {
                told(&answer);
                then();
            }
            Err(error) => web_sys::console::warn_1(&error.into()),
        }
    });
}

/// Keeps what the gateway answered about some items: `{"<id>": {"name", "resource", "description": […],
/// "facts": [[key, value], …], "unidentified": {"name", "resource", "description"} or null, "slots"}}`
/// (`tables/items.rs`).
fn told(answer: &wasm_bindgen::JsValue) {
    for (id, about) in entries::<u32>(answer) {
        if let (Some(name), Some(resource)) = (text_of(&about, "name"), text_of(&about, "resource")) {
            ITEMS.with_borrow_mut(|items| items.insert(id, (name, resource)));
        }
        let facts = list(&field(&about, "facts")).iter().filter_map(|fact| <[String; 2]>::try_from(texts(fact)).ok()).map(|[key, value]| (key, value)).collect();
        ABOUT.with_borrow_mut(|all| all.insert(id, (texts(&field(&about, "description")), facts)));
        let unknown = field(&about, "unidentified");
        if let (true, Some(name), Some(resource)) = (unknown.is_object(), text_of(&unknown, "name"), text_of(&unknown, "resource")) {
            UNIDENTIFIED.with_borrow_mut(|all| all.insert(id, (name, resource, texts(&field(&unknown, "description")))));
        }
        if let Some(slots) = number(&about, "slots").filter(|slots| *slots > 0.0) {
            SLOTS.with_borrow_mut(|all| all.insert(id, slots as u8));
        }
    }
}

/// Forgets the items' texts: after the language changed they are asked for anew.
pub(super) fn forget() {
    ITEMS.with_borrow_mut(HashMap::clear);
    ABOUT.with_borrow_mut(HashMap::clear);
    UNIDENTIFIED.with_borrow_mut(HashMap::clear);
    SLOTS.with_borrow_mut(HashMap::clear);
    ITEMS_ASKED.with_borrow_mut(std::collections::HashSet::clear);
}
