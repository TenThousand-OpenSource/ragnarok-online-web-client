//! The states a character is in (its buffs and debuffs): the picture of each and what is said of it.

use super::*;

/// What the client's Lua files tell of the states (`Lua Files\StateIcon\*`, loaded by its `CSession` into the
/// script it asks): `{"icons": {"<state>": [kind, "file"]}, "texts": {"<state>": {"lines": [[text, [r, g, b]],
/// …], "time": line}}}`; a line in white has no colour written.
///
/// The files' own functions are asked, as the client asks them: `GetEFSTImgFileName(state, kind)` for the
/// picture (the client shows the states kind by kind, 0 to 4, so the first kind that has one is kept),
/// `GetStateIconDescript(state, line)` for each line of the description and its colour, from line 1 until an
/// empty one, and `GetTimeLimitInfo(state)` for whether a time is written into it and on which line (0: none).
pub fn statuses(grf: &Grf) -> serde_json::Value {
    let lua = state();
    run_in(&lua, grf, &["stateicon\\efstids", "stateicon\\stateiconinfo", "stateicon\\stateiconinfo_f", "stateicon\\stateiconimginfo"]);
    let globals = lua.globals();
    let states: Vec<i64> = names_of(&globals, "EFST_IDs");
    let text = |value: &Value| match value {
        Value::String(text) => korean(&text.as_bytes()),
        _ => String::new(),
    };
    let mut icons = serde_json::Map::new();
    if let Ok(picture) = globals.get::<mlua::Function>("GetEFSTImgFileName") {
        for state in &states {
            let found = (0..5).find_map(|kind| picture.call::<Value>((*state, kind)).ok().map(|file| text(&file)).filter(|file| !file.is_empty()).map(|file| (kind, file)));
            if let Some((kind, file)) = found {
                icons.insert(state.to_string(), serde_json::json!([kind, file]));
            }
        }
    }
    let mut texts = serde_json::Map::new();
    if let (Ok(described), Ok(timed)) = (globals.get::<mlua::Function>("GetStateIconDescript"), globals.get::<mlua::Function>("GetTimeLimitInfo")) {
        for state in &states {
            let lines: Vec<serde_json::Value> = (1..100)
                .map_while(|line| {
                    let (words, red, green, blue) = described.call::<(Value, Option<i64>, Option<i64>, Option<i64>)>((*state, line)).ok()?;
                    let words = text(&words);
                    let colour = [red.unwrap_or(0), green.unwrap_or(0), blue.unwrap_or(0)];
                    // Most lines are white: those are sent without their colour.
                    (!words.is_empty()).then(|| if colour == [255; 3] { serde_json::json!([words]) } else { serde_json::json!([words, colour]) })
                })
                .collect();
            if lines.is_empty() {
                continue;
            }
            let (has, line) = timed.call::<(Option<i64>, Option<i64>)>(*state).unwrap_or((None, None));
            let time = if has.unwrap_or(0) != 0 { line.unwrap_or(0).max(0) } else { 0 };
            texts.insert(state.to_string(), serde_json::json!({ "lines": lines, "time": time }));
        }
    }
    serde_json::json!({ "icons": icons, "texts": texts })
}

/// The numbers a table of the globals gives to its names (`EFST_IDs`: each state's number by its name).
fn names_of(globals: &Table, table: &str) -> Vec<i64> {
    let Ok(table) = globals.get::<Table>(table) else { return Vec::new() };
    let mut numbers: Vec<i64> = table
        .pairs::<Value, Value>()
        .flatten()
        .filter_map(|(_, number)| whole(&number))
        .filter(|number| *number >= 0)
        .collect();
    numbers.sort_unstable();
    numbers.dedup();
    numbers
}
