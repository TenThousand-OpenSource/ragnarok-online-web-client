//! The lines of Frost Joke and Scream (`crate::song_talk`): the two files of the client's data they are drawn
//! from, read once at the start so that the first joke already has its line.

use super::*;

thread_local! {
    /// The files read, by the effect that says a line of each.
    static TABLES: RefCell<HashMap<u32, String>> = RefCell::new(HashMap::new());
}

/// Fetches the two files; each is there some time after.
pub(super) fn fetch() {
    for effect in [crate::song_talk::EF_TALK_FROSTJOKE, crate::song_talk::EF_TALK_SCREAM] {
        let Some((file, _)) = crate::song_talk::table(effect) else { continue };
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(bytes) = crate::web::fetch_asset(file).await {
                TABLES.with_borrow_mut(|tables| tables.insert(effect, String::from_utf8_lossy(&bytes).into_owned()));
            }
        });
    }
}

/// A line for an effect of talk to say, drawn by lot as the client draws it; none for any other effect, while
/// its file is not read, or if the line drawn is empty.
pub fn song_line(effect: u32) -> Option<String> {
    let (_, lines) = crate::song_talk::table(effect)?;
    let number = (js_sys::Math::random() * f64::from(lines)) as u32 + 1;
    TABLES.with_borrow(|tables| tables.get(&effect).map(|table| crate::song_talk::line(table, number))).filter(|line| !line.is_empty())
}
