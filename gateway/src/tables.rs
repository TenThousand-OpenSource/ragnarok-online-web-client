//! The client's own tables, read out of its files for the browser: which sprite a job or a headgear is, what
//! a skill, a map, a quest is called, and the rest `for_browser` puts together. Each kind of table has its
//! file in `tables/`; this one has what they share.
//!
//! Most of the client's tables are Lua 5.1 bytecode compiled for the 32-bit client. `lunify` rewrites it for
//! the Lua this program embeds, which then simply runs it; the tables are what the files leave in the globals.
//! Shared here: a Lua state as the files need it (`state`), running a file in it (`exec`, `run`, `run_in`,
//! `beside`), walking what it left (`whole`, `text`, `text_only`, `lines`, `names`, `numbered`, `json_of`), the texts'
//! encodings (`korean`, `latin`), the client's `#` tables (`hash_fields`), and a JSON object keyed by numbers
//! in their order (`by_number`).

use std::collections::BTreeMap;

use mlua::{Lua, Table, Value};

use crate::grf::Grf;

// Each kind of table has its file here; what is common to them (running the client's Lua, reading its Korean,
// putting the browser's tables together) is in this one.
mod ai;
mod aura;
mod cards;
mod figures;
mod guide;
mod interface;
mod items;
mod jobs;
mod maps;
mod pets;
mod quests;
mod scenery;
mod commands;
mod skills;
mod statuses;
mod tips;
mod worldmap;

#[cfg(test)]
mod tests;

pub use ai::scripts as ai_scripts;
pub use interface::{chat_tabs, emotions, hotkeys, options};
pub use items::{translated, About, Items, Translated, Unidentified};
pub use skills::skill_texts;
use skills::{job_inherits, skill_tree, skills};

/// What Lua itself puts in the globals.
const BUILT_IN: [&str; 30] = [
    "print", "pairs", "ipairs", "type", "next", "tostring", "tonumber", "assert", "collectgarbage", "dofile", "error", "gcinfo", "getfenv", "getmetatable", "load",
    "loadfile", "loadstring", "module", "newproxy", "pcall", "rawequal", "rawget", "rawset", "require", "select", "setfenv", "setmetatable", "unpack", "xpcall", "_VERSION",
];

const ROOT: &str = "data\\luafiles514\\lua files\\";

/// Runs the client's Lua files, in order, in one Lua state. A file that is missing or does not run is skipped
/// with a note: the tables it would have filled stay empty.
pub fn run(grf: &Grf, files: &[&str]) -> Lua {
    let lua = state();
    run_in(&lua, grf, files);
    lua
}

/// A Lua state for the client's files. They only build tables: they get the string, table and math libraries,
/// and neither files nor the system.
fn state() -> Lua {
    Lua::new_with(mlua::StdLib::TABLE | mlua::StdLib::STRING | mlua::StdLib::MATH, mlua::LuaOptions::default()).expect("a Lua state")
}

/// Runs one of the client's Lua files in a state: bytecode, as most are (rewritten for this Lua first), or
/// plain text, as the translated ones are. `name` is what an error calls the file, if it is given one.
fn exec(lua: &Lua, bytes: &[u8], name: Option<&str>) -> Result<(), String> {
    let chunk = if bytes.starts_with(b"\x1bLua") {
        let bytecode = lunify::unify(bytes, &lunify::Format::default(), &lunify::Settings::default()).map_err(|error| format!("{error:?}"))?;
        lua.load(bytecode).set_mode(mlua::ChunkMode::Binary)
    } else {
        lua.load(bytes).set_mode(mlua::ChunkMode::Text)
    };
    match name {
        Some(name) => chunk.set_name(name).exec(),
        None => chunk.exec(),
    }
    .map_err(|error| error.to_string())
}

/// Makes every name a file uses and nothing has defined stand for itself, as a text: for a file that names
/// things by the client's own constants (`optioninfo\gamesettings`), which this program does not have.
fn names_stand_for_themselves(lua: &Lua) -> Result<(), String> {
    lua.load("setmetatable(_G, { __index = function(_, name) return name end })").exec().map_err(|error| error.to_string())
}

/// Runs more of the client's Lua files in a state that may hold others' tables already.
fn run_in(lua: &Lua, grf: &Grf, files: &[&str]) {
    for file in files {
        // A name may carry its own folder under `lua files`; the others are in `datainfo`.
        // …and one that starts with a backslash is right under `lua files`.
        let path = match file.strip_prefix('\\') {
            Some(top) => format!("{ROOT}{top}.lub"),
            None if file.contains('\\') => format!("{ROOT}{file}.lub"),
            None => format!("{ROOT}datainfo\\{file}.lub"),
        };
        let outcome = grf.read(&path).map_err(|error| error.to_string()).and_then(|bytes| exec(lua, &bytes, Some(*file)));
        if let Err(error) = outcome {
            eprintln!("tables: {file}.lub: {error}");
        }
    }
}

/// A Lua value as JSON: a table with only 1, 2, 3… for keys as a list, any other as an object; text read as
/// the client's Korean; functions and the like as nothing.
pub fn json_of(value: &Value) -> serde_json::Value {
    match value {
        Value::Boolean(flag) => (*flag).into(),
        Value::Integer(number) => (*number).into(),
        Value::Number(number) => if number.fract() == 0.0 { (*number as i64).into() } else { (*number).into() },
        Value::String(text) => korean(&text.as_bytes()).into(),
        Value::Table(table) => {
            let length = table.raw_len();
            let pairs: Vec<(Value, Value)> = table.pairs::<Value, Value>().flatten().collect();
            if length > 0 && pairs.len() == length {
                (1..=length).map(|at| json_of(&table.raw_get::<Value>(at).unwrap_or(Value::Nil))).collect::<Vec<_>>().into()
            } else {
                pairs
                    .iter()
                    .filter_map(|(key, value)| {
                        let key = match key {
                            Value::String(key) => korean(&key.as_bytes()),
                            Value::Integer(key) => key.to_string(),
                            Value::Number(key) => key.to_string(),
                            _ => return None,
                        };
                        Some((key, json_of(value)))
                    })
                    .collect::<serde_json::Map<String, serde_json::Value>>()
                    .into()
            }
        }
        _ => serde_json::Value::Null,
    }
}

/// The names of the global tables a state holds and how many entries each has, for looking around.
pub fn describe(lua: &Lua) -> Vec<String> {
    let mut lines = Vec::new();
    for pair in lua.globals().pairs::<String, Value>() {
        let Ok((name, value)) = pair else { continue };
        match value {
            Value::Table(table) if !["_G", "package", "string", "table", "math", "coroutine"].contains(&name.as_str()) => {
                let entries: Vec<String> = table.pairs::<Value, Value>().flatten().take(4).map(|(key, value)| format!("{} = {}", show(&key), show(&value))).collect();
                lines.push(format!("{name}: {} entries; {}", table.pairs::<Value, Value>().count(), entries.join(", ")));
            }
            Value::Function(_) if !BUILT_IN.contains(&name.as_str()) => lines.push(format!("{name}: function")),
            _ => {}
        }
    }
    lines.sort();
    lines
}

fn show(value: &Value) -> String {
    match value {
        Value::String(text) => format!("{:?}", korean(&text.as_bytes())),
        Value::Integer(number) => number.to_string(),
        Value::Number(number) => number.to_string(),
        Value::Table(_) => "{…}".to_owned(),
        other => other.type_name().to_owned(),
    }
}

/// The client's strings are EUC-KR. Those of converted bytecode arrive with every byte widened to a character
/// of its own (as Latin-1 read into UTF-8), which is undone first.
pub fn korean(bytes: &[u8]) -> String {
    let narrowed: Option<Vec<u8>> = std::str::from_utf8(bytes).ok().and_then(|text| text.chars().map(|character| u8::try_from(u32::from(character)).ok()).collect());
    encoding_rs::EUC_KR.decode_without_bom_handling(narrowed.as_deref().unwrap_or(bytes)).0.into_owned()
}

/// A text that is UTF-8, or else Windows-1252: the translated files beside the archives are plain text, and
/// the few bytes beyond ASCII in them are Latin letters and signs.
fn latin(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_owned(),
        Err(_) => encoding_rs::WINDOWS_1252.decode_without_bom_handling(bytes).0.into_owned(),
    }
}

/// A whole number of a Lua table, which keeps some of its whole numbers as numbers with a fraction of nought;
/// none for anything that is no number.
fn whole(value: &Value) -> Option<i64> {
    match value {
        Value::Integer(number) => Some(*number),
        Value::Number(number) => Some(*number as i64),
        _ => None,
    }
}

/// The text a Lua value is, read as the client's Korean; none of what is no text: a number is not taken for
/// one, as `text` takes it.
fn text_only(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(korean(&text.as_bytes())),
        _ => None,
    }
}

/// The text in a field of a Lua table, read as the client's Korean; none when the field is no text. (Lua
/// gives a number in the field as its text: `text_only` for where that must not be.)
fn text(table: &Table, field: &str) -> Option<String> {
    table.get::<Option<mlua::String>>(field).ok().flatten().map(|text| korean(&text.as_bytes()))
}

/// The texts a Lua table lists (1, 2, 3…), each read as the client's Korean, up to the first that is no text.
fn lines(table: &Table) -> Vec<String> {
    table.sequence_values::<mlua::String>().flatten().map(|line| korean(&line.as_bytes())).collect()
}

/// The entries of one of the globals' tables that are tables themselves, each by its number and as JSON, in
/// the order of their numbers. The table is gone through an entry at a time: the quests' has some eleven
/// thousand, more than can be held at once the way `json_of` holds a table.
fn numbered(lua: &Lua, name: &str) -> Vec<(u64, serde_json::Value)> {
    let Ok(table) = lua.globals().get::<Table>(name) else { return Vec::new() };
    let mut entries: Vec<(u64, serde_json::Value)> = table
        .pairs::<Value, Value>()
        .flatten()
        .filter_map(|(number, entry)| {
            let number = match number {
                Value::Integer(number) => u64::try_from(number).ok()?,
                Value::Number(number) if number >= 0.0 => number as u64,
                _ => return None,
            };
            matches!(entry, Value::Table(_)).then(|| (number, json_of(&entry)))
        })
        .collect();
    entries.sort_by_key(|(number, _)| *number);
    entries
}

/// The entries of a JSON object whose keys are numbers (what `json_of` makes of a Lua table that is no plain
/// list), in the order of their numbers; a key that is no number is passed by, and what is no object has none.
fn by_number(object: &serde_json::Value) -> Vec<(i64, &serde_json::Value)> {
    let mut entries: Vec<(i64, &serde_json::Value)> = object.as_object().map(|entries| entries.iter().filter_map(|(at, entry)| Some((at.parse().ok()?, entry))).collect()).unwrap_or_default();
    entries.sort_by_key(|(at, _)| *at);
    entries
}

/// The fields of one of the client's `#` tables, line by line and in order: `4001#Lucky#` gives `4001`,
/// `Lucky`. An empty line and one that begins a comment (`//`) give nothing.
pub(crate) fn hash_fields(text: &str) -> Vec<Vec<&str>> {
    text.lines().map(str::trim).filter(|line| !line.is_empty() && !line.starts_with("//")).map(|line| line.trim_end_matches('#').split('#').collect()).collect()
}

/// A table of the globals as numbers to strings; entries of other types are left out.
pub fn names(lua: &Lua, table: &str) -> BTreeMap<i64, String> {
    let Ok(table) = lua.globals().get::<Table>(table) else { return BTreeMap::new() };
    table
        .pairs::<Value, Value>()
        .flatten()
        .filter_map(|(key, value)| match (key, value) {
            (Value::Integer(key), Value::String(text)) => Some((key, korean(&text.as_bytes()))),
            (Value::Number(key), Value::String(text)) => Some((key as i64, korean(&text.as_bytes()))),
            _ => None,
        })
        .collect()
}

/// What the browser client needs of the client's tables, as JSON:
/// `{"jobs": {"<id>": {"name": …}}, "headgear": {"<view id>": …}, "skills": {…}}`: what each job is called, the
/// name each headgear's sprite files end in, and the skills (see `skills`).
pub fn for_client(grf: &Grf) -> serde_json::Value {
    let (lua, jobs, headgear, sprites, weapons, weapon_hits) = jobs::jobs(grf);
    let (pets, pet_acts) = pets::pets(&lua, grf);
    serde_json::json!({ "jobs": jobs, "headgear": headgear, "skills": skills(grf), "sprites": sprites, "weapons": weapons, "weapon_hits": weapon_hits, "tree": skill_tree(grf), "inherits": job_inherits(grf), "pets": pets, "pet_acts": pet_acts, "emotions": emotions(grf), "messages": messages(grf), "hotkeys": hotkeys(grf), "options": options(grf) })
}

/// The client's message table (`msgstringtable.txt`), line by line: what the client says, by the numbers its code
/// knows the lines by. (The client of this version fills its table from `msgstringtable.csv`, by the names of
/// the lines, into the same numbers: a name's number is its place in the list of names compiled into the
/// client, `messageNames` of rodecomp's `MessageTable.cpp`, and the two files agree line for line but for the
/// wording of some fifty.)
pub fn messages(grf: &Grf) -> Vec<String> {
    grf.read("data\\msgstringtable.txt").map(|bytes| korean(&bytes).lines().map(|line| line.trim_end_matches('#').to_owned()).collect()).unwrap_or_default()
}

/// Runs one of the client's Lua files that lie beside the archives, not in them (`System\\mapInfo_true.lub` and
/// the like): the first of `files` (paths under the client's folder) that is there. Such a file is either
/// bytecode, as the archives' are, or plain text, as the translations are.
fn beside(client_dir: &std::path::Path, files: &[&str]) -> Option<Lua> {
    let (file, bytes) = files.iter().find_map(|file| Some((file, std::fs::read(client_dir.join(file)).ok()?)))?;
    let lua = state();
    if let Err(error) = exec(&lua, &bytes, None) {
        eprintln!("tables: {file}: {error}");
    }
    Some(lua)
}

/// Everything the browser is given at once (`/tables.json`): the tables out of the archives (`for_client`), with
/// those that come from beside the archives and from the server's database added, each under its own name: what
/// the client calls each map, who stands where in the towns, the Adventure Guide.
pub fn for_browser(grf: &Grf, client_dir: &std::path::Path, database: &std::path::Path) -> serde_json::Value {
    let mut tables = for_client(grf);
    if let Some(tables) = tables.as_object_mut() {
        tables.insert("chat_tabs".to_owned(), chat_tabs(grf));
        tables.insert("maps".to_owned(), maps::maps(client_dir));
        tables.insert("towns".to_owned(), maps::towns(client_dir));
        tables.insert("guide".to_owned(), guide::guide(grf, database, client_dir));
        tables.insert("statuses".to_owned(), statuses::statuses(grf));
        tables.insert("commands".to_owned(), commands::commands(grf));
        tables.insert("quests".to_owned(), quests::quests(client_dir, database));
        tables.insert("cards".to_owned(), cards::cards(grf));
        tables.insert("sky".to_owned(), scenery::sky(grf));
        tables.insert("figures".to_owned(), figures::figures(grf));
        tables.insert("worldmap".to_owned(), worldmap::worldmap(grf));
        tables.insert("tips".to_owned(), tips::tips(client_dir));
        tables.insert("aura".to_owned(), aura::aura(grf));
        // Which rules the server plays by, told by the database that goes with them: rAthena's `db/re` is
        // Renewal's, `db/pre-re` the classic game's. The client, which is one for both, asks where they differ
        // in what a player may do.
        tables.insert("renewal".to_owned(), database.file_name().is_some_and(|folder| folder == "re").into());
    }
    tables
}
