//! The castles guilds hold, as the client's own table has them (the gateway's `tables/castles.rs`, out of
//! `AgitConfig.lub` and `AgitConfig_F.lub`).

use super::*;

/// What the client's table says of a castle: its name, and the zeny going there costs, usually and during a
/// siege (the client's `get_agit_name`, and `get_agit_costzeny` asked both ways).
#[derive(Clone, Default, PartialEq)]
pub struct Castle {
    pub name: String,
    pub cost: u32,
    pub siege_cost: u32,
}

thread_local! {
    /// The castles by the number the map server names each by; with a gateway older than the table, none.
    static CASTLES: RefCell<HashMap<u8, Castle>> = RefCell::new(HashMap::new());
}

/// A castle by its number; none for one the client's table does not name (the client then leaves it out of
/// the guild window's list, and asks nothing of it).
pub fn castle(number: u8) -> Option<Castle> {
    CASTLES.with_borrow(|castles| castles.get(&number).cloned())
}

/// Reads the castles out of the gateway's tables: `{"<number>": {"name": "…", "cost": 100, "siege": 100000}}`.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    let castles = entries::<u8>(&field(tables, "castles"))
        .into_iter()
        .filter_map(|(number, about)| {
            let zeny = |name: &str| number_of(&about, name);
            Some((number, Castle { name: text_of(&about, "name").filter(|name| !name.is_empty())?, cost: zeny("cost"), siege_cost: zeny("siege") }))
        })
        .collect();
    CASTLES.set(castles);
}

/// A sum of zeny in a field of an entry; nothing when the field has none.
fn number_of(about: &wasm_bindgen::JsValue, name: &str) -> u32 {
    number(about, name).unwrap_or(0.0).max(0.0) as u32
}
