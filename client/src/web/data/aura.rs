//! What the client's settings say of the aura of the highest level (the gateway's `tables/aura.rs`): the
//! table `crate::aura` decides by.

use super::*;

thread_local! {
    /// The table as the gateway gave it; with a gateway older than it, nothing: no aura shows then.
    static TABLE: RefCell<crate::aura::Table> = RefCell::new(crate::aura::Table::default());
}

/// The highest level and the effects of its aura, as the client's `ExternalSettings` has them.
pub fn aura_table() -> crate::aura::Table {
    TABLE.with_borrow(|table| *table)
}

/// Takes its table out of what the gateway gave.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    let aura = field(tables, "aura");
    let level = |name: &str| number(&aura, name).unwrap_or(0.0) as u16;
    let pair = |name: &str| -> [u32; 2] {
        let pair = list(&field(&aura, name));
        [0, 1].map(|index| pair.get(index).and_then(|effect| effect.as_f64()).unwrap_or(0.0) as u32)
    };
    TABLE.set(crate::aura::Table { highest: level("highest"), homunculus: level("homunculus"), plain: pair("plain"), baby: pair("baby"), high: pair("high") });
}
