//! The words the client does not let be written (the gateway's `tables/manner.rs`, the client's
//! `manner.txt`), asked as the client's `g_forbiddenWords` is: whether a text has one, and the text with stars
//! in their places. The rule is `crate::manner`.

use super::*;

thread_local! {
    static WORDS: RefCell<crate::manner::Words> = RefCell::new(crate::manner::Words::default());
}

/// Whether a text has a word that is not let be written (`ForbiddenWords::Contains`).
pub fn foul(text: &str) -> bool {
    WORDS.with_borrow(|words| !words.allows(text))
}

/// A text with stars in the place of every such word (`ForbiddenWords::Mask`).
pub fn starred(text: &str) -> String {
    WORDS.with_borrow(|words| words.masked(text))
}

/// Takes the words out of what the gateway gave. An older gateway gives none: then every text is let be.
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    WORDS.set(crate::manner::Words::new(texts(&field(tables, "manner"))));
}
