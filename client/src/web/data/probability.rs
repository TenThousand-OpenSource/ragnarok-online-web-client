//! The probabilities: what the menu's icon of them says, asks and opens (the gateway's
//! `tables/probability.rs`: three lines of the client's message table).

use super::*;

thread_local! {
    /// What the icon says when pointed at, what it asks when pressed, and the address a yes opens.
    static LINES: RefCell<[String; 3]> = const { RefCell::new([String::new(), String::new(), String::new()]) };
}

/// What the icon says when pointed at; nothing until the tables are there.
pub fn probability_title() -> String {
    LINES.with_borrow(|lines| lines[0].clone())
}

/// What the icon asks when pressed.
pub fn probability_asks() -> String {
    LINES.with_borrow(|lines| lines[1].clone())
}

/// The line of the address a yes opens, as the client's table has it.
pub fn probability_address() -> String {
    LINES.with_borrow(|lines| lines[2].clone())
}

/// Takes the lines out of what the gateway gave.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    let told = field(tables, "probability");
    LINES.set(["title", "asks", "address"].map(|name| text_of(&told, name).unwrap_or_default()));
}
