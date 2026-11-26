//! The cash shop: the banners of the strip under its title.

use super::*;

thread_local! {
    /// The banners, in the order the client's file adds them (`datainfo\tb_cashshop_banner.lub`): each a
    /// picture of the folder `cashshop` and the address its page is at.
    static BANNERS: RefCell<std::rc::Rc<Vec<(String, String)>>> = RefCell::new(std::rc::Rc::new(Vec::new()));
}

/// The banners of the cash shop.
pub fn cashshop_banners() -> std::rc::Rc<Vec<(String, String)>> {
    BANNERS.with_borrow(Clone::clone)
}

/// Takes the banners out of what the gateway gave (`tables/cashshop.rs`).
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    let banners = list(&field(&field(tables, "cashshop"), "banners"))
        .iter()
        .filter_map(|banner| match texts(banner).as_slice() {
            [image, url] => Some((image.clone(), url.clone())),
            _ => None,
        })
        .collect();
    BANNERS.set(std::rc::Rc::new(banners));
}
