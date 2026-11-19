//! The scripts that move a homunculus and a mercenary (`crate::ai`): the Lua files of the client's `AI`
//! folder, as the gateway gives them (`gateway/src/tables/ai.rs`). They are asked for when the player first
//! has a homunculus or a mercenary out, not at the start: most players never have one.

use std::cell::Cell;

use super::*;
use crate::ai::Files;

thread_local! {
    /// The client's own scripts, and those with the player's own beside them (`USER_AI`), once read.
    static SCRIPTS: RefCell<[Option<Files>; 2]> = const { RefCell::new([None, None]) };
    /// Which of the two were asked for.
    static ASKED: Cell<[bool; 2]> = const { Cell::new([false, false]) };
}

/// The scripts: the client's own, or with `user` those and the player's own. None until they are read; the
/// first asking fetches them, and they are there some time after.
pub fn ai_scripts(user: bool) -> Option<Files> {
    let which = usize::from(user);
    if let Some(files) = SCRIPTS.with_borrow(|scripts| scripts[which].clone()) {
        return Some(files);
    }
    let mut asked = ASKED.get();
    if !asked[which] {
        asked[which] = true;
        ASKED.set(asked);
        wasm_bindgen_futures::spawn_local(async move {
            match fetch_json(if user { "/tables/ai-user.json" } else { "/tables/ai.json" }).await {
                // A text has a character for each byte of its file.
                Ok(files) => {
                    let files = entries::<String>(&files)
                        .into_iter()
                        .filter_map(|(path, text)| Some((path, text.as_string()?.chars().map(|byte| byte as u8).collect())))
                        .collect();
                    SCRIPTS.with_borrow_mut(|scripts| scripts[which] = Some(files));
                }
                Err(error) => web_sys::console::error_1(&format!("ai scripts: {error}").into()),
            }
        });
    }
    None
}
