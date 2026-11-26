//! The reputation: the client's reputations and the groups its window sorts them by (the gateway's
//! `tables/reputation.rs`).

use super::*;
use crate::reputation::{Group, Repute, Visibility};

thread_local! {
    /// The reputations and their groups, each in the order of its numbers (`ContentData\Repute\*.bson`).
    static TABLES: RefCell<std::rc::Rc<(Vec<Repute>, Vec<Group>)>> = RefCell::new(std::rc::Rc::new((Vec::new(), Vec::new())));
}

/// The client's reputations and their groups.
pub fn reputation() -> std::rc::Rc<(Vec<Repute>, Vec<Group>)> {
    TABLES.with_borrow(Clone::clone)
}

/// Takes the tables out of what the gateway gave.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    let told = field(tables, "reputation");
    let whole = |object: &wasm_bindgen::JsValue, name: &str| number(object, name).unwrap_or(0.0).max(0.0) as u64;
    let reputes = list(&field(&told, "reputes"))
        .into_iter()
        .map(|one| Repute {
            id: whole(&one, "id"),
            name: text_of(&one, "name").unwrap_or_default(),
            most: whole(&one, "most"),
            least: whole(&one, "least"),
            visibility: Visibility::of(&text_of(&one, "seen").unwrap_or_default()),
        })
        .collect();
    let groups = list(&field(&told, "groups"))
        .into_iter()
        .map(|group| Group {
            id: whole(&group, "id"),
            name: text_of(&group, "name").unwrap_or_default(),
            reputes: list(&field(&group, "reputes")).iter().filter_map(|id| id.as_f64().map(|id| id as u64)).collect(),
        })
        .collect();
    TABLES.set(std::rc::Rc::new((reputes, groups)));
}
