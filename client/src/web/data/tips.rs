//! The tip box: the client's tips, each with its title and its pages.

use super::*;

thread_local! {
    /// The tips, in the order of their numbers (the client's `System\tipbox.lub`).
    static TIPS: RefCell<std::rc::Rc<Vec<Tip>>> = RefCell::new(std::rc::Rc::new(Vec::new()));
}

/// One tip: its number, its title (what stands after a `#` in it is the group it belongs to) and its pages,
/// with the client's marks of colour and its links in them.
pub struct Tip {
    pub id: u32,
    pub title: String,
    pub pages: Vec<String>,
}

/// The client's tips.
pub fn tips() -> std::rc::Rc<Vec<Tip>> {
    TIPS.with_borrow(Clone::clone)
}

/// Takes the tips out of what the gateway gave (`tables/tips.rs`).
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    let tips = list(&field(tables, "tips"))
        .into_iter()
        .map(|tip| Tip {
            id: number(&tip, "id").unwrap_or(0.0) as u32,
            title: text_of(&tip, "title").unwrap_or_default(),
            pages: texts(&field(&tip, "pages")),
        })
        .collect();
    TIPS.set(std::rc::Rc::new(tips));
}
