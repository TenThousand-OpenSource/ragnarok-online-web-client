//! The states a character is in (buffs and debuffs): the picture of each, and what the client says of it.

use super::*;

/// What the client says of a state when its picture is pointed at: the lines with their colours, and which of
/// them (counted from 1; 0 for none) has the time left written into it.
#[derive(Clone, Default)]
pub struct StatusText {
    pub lines: Vec<(String, [u8; 3])>,
    pub time: usize,
}

thread_local! {
    /// The picture of each state in the client's Lua files (`StateIconImgInfo`): the kind it is listed under,
    /// and the file, as its `GetEFSTImgFileName` gives it.
    static ICONS: RefCell<HashMap<u32, (u8, String)>> = RefCell::new(HashMap::new());
    /// The description of each state (`StateIconInfo`).
    static TEXTS: RefCell<HashMap<u32, StatusText>> = RefCell::new(HashMap::new());
}

/// The picture of a state listed under a kind, as the client finds it (`CSession::GetEFSTImgFileName`): its Lua
/// files are asked first, then what it has compiled into it (`crate::statuses`). A file under `data\texture`,
/// written with forward strokes.
pub fn status_icon(state: u32, kind: u8, taekwon: bool) -> Option<String> {
    ICONS.with_borrow(|icons| crate::statuses::icon(icons.get(&state).map(|(of, file)| (*of, file.as_str())), state, kind, taekwon))
}

/// What the client says of a state.
pub fn status_text(state: u32) -> Option<StatusText> {
    TEXTS.with_borrow(|texts| texts.get(&state).cloned())
}

/// Takes its tables out of what the gateway gave (`tables/statuses.rs`).
pub(super) fn read(tables: &wasm_bindgen::JsValue) {
    let statuses = field(tables, "statuses");
    let icons = entries::<u32>(&field(&statuses, "icons")).into_iter().filter_map(|(state, icon)| {
        let icon: js_sys::Array = icon.dyn_into().ok()?;
        Some((state, (icon.get(0).as_f64()? as u8, icon.get(1).as_string()?)))
    });
    ICONS.set(icons.collect());
    let texts = entries::<u32>(&field(&statuses, "texts")).into_iter().filter_map(|(state, text)| {
        let lines: js_sys::Array = field(&text, "lines").dyn_into().ok()?;
        let lines = lines.iter().filter_map(|line| {
            let line: js_sys::Array = line.dyn_into().ok()?;
            // A line without a colour is white.
            let colour = line.get(1).dyn_into::<js_sys::Array>().map_or([255; 3], |colour| [0, 1, 2].map(
                |part| colour.get(part).as_f64().unwrap_or(255.0) as u8
            ));
            Some((line.get(0).as_string()?, colour))
        });
        Some((state, StatusText { lines: lines.collect(), time: number(&text, "time").unwrap_or(0.0) as usize }))
    });
    TEXTS.set(texts.collect());
}
