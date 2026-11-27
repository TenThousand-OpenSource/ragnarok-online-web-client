//! The old party booking's list of places, its help and the game's highest level as its windows take it (the
//! gateway's `tables/booking.rs`; the client's `party_booking_config.lub`).

use super::*;
use crate::booking::Region;

thread_local! {
    static REGIONS: RefCell<std::rc::Rc<Vec<Region>>> = RefCell::new(std::rc::Rc::new(Vec::new()));
    static HELP: RefCell<String> = const { RefCell::new(String::new()) };
    static HIGHEST: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}

/// The regions in the client's order, the first of them "all maps", each with its maps.
pub fn booking_regions() -> std::rc::Rc<Vec<Region>> {
    REGIONS.with_borrow(Clone::clone)
}

/// The lines of the booking's help.
pub fn booking_help() -> String {
    HELP.with_borrow(Clone::clone)
}

/// The game's highest level, as the booking's windows bound a level typed (the client's `m_maxLevel`).
pub fn booking_highest() -> i32 {
    HIGHEST.get()
}

/// Takes them out of what the gateway gave. An older gateway gives none.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    let booking = field(tables, "booking");
    let regions = list(&field(&booking, "regions"))
        .into_iter()
        .map(|region| Region {
            name: text_of(&region, "name").unwrap_or_default(),
            maps: list(&field(&region, "maps"))
                .into_iter()
                .map(|map| {
                    let colour = list(&field(&map, "colour"));
                    let part = |place: usize| colour.get(place).and_then(|part| part.as_f64()).unwrap_or(0.0) as u8;
                    (text_of(&map, "name").unwrap_or_default(), [part(0), part(1), part(2)])
                })
                .collect(),
        })
        .collect();
    REGIONS.set(std::rc::Rc::new(regions));
    HELP.set(text_of(&booking, "help").unwrap_or_default());
    HIGHEST.set(number(&booking, "highest").unwrap_or(0.0) as i32);
}
