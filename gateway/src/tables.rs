//! The client's own tables (which sprite a job or a headgear is), read out of its Lua files.
//!
//! The files are Lua 5.1 bytecode compiled for the 32-bit client. `lunify` rewrites it for the Lua this program
//! embeds, which then simply runs it; the tables are what the files leave in the globals.

use std::collections::BTreeMap;

use mlua::{Lua, Table, Value};

use crate::grf::Grf;

// Each kind of table has its file here; what is common to them (running the client's Lua, reading its Korean,
// putting the browser's tables together) is in this one.
mod guide;
mod interface;
mod items;
mod maps;
mod skills;

pub use interface::{chat_tabs, emotions, hotkeys, options};
pub use items::{translated, About, Items, Translated, Unidentified};
pub use skills::skill_texts;
use skills::{skill_tree, skills};

/// What Lua itself puts in the globals.
const BUILT_IN: [&str; 30] = [
    "print", "pairs", "ipairs", "type", "next", "tostring", "tonumber", "assert", "collectgarbage", "dofile", "error", "gcinfo", "getfenv", "getmetatable", "load",
    "loadfile", "loadstring", "module", "newproxy", "pcall", "rawequal", "rawget", "rawset", "require", "select", "setfenv", "setmetatable", "unpack", "xpcall", "_VERSION",
];

const ROOT: &str = "data\\luafiles514\\lua files\\";

/// Runs the client's Lua files, in order, in one Lua state. A file that is missing or does not run is skipped
/// with a note: the tables it would have filled stay empty.
pub fn run(grf: &Grf, files: &[&str]) -> Lua {
    // The files only build tables: they get the string, table and math libraries, and neither files nor the system.
    let lua = Lua::new_with(mlua::StdLib::TABLE | mlua::StdLib::STRING | mlua::StdLib::MATH, mlua::LuaOptions::default()).expect("a Lua state");
    run_in(&lua, grf, files);
    lua
}

fn run_in(lua: &Lua, grf: &Grf, files: &[&str]) {
    for file in files {
        // A name may carry its own folder under `lua files`; the others are in `datainfo`.
        // …and one that starts with a backslash is right under `lua files`.
        let path = match file.strip_prefix('\\') {
            Some(top) => format!("{ROOT}{top}.lub"),
            None if file.contains('\\') => format!("{ROOT}{file}.lub"),
            None => format!("{ROOT}datainfo\\{file}.lub"),
        };
        let outcome = grf.read(&path).map_err(|error| error.to_string()).and_then(|bytes| {
            // Most files are bytecode; the translated ones are Lua source.
            let chunk = if bytes.starts_with(b"\x1bLua") {
                let bytecode = lunify::unify(&bytes, &lunify::Format::default(), &lunify::Settings::default()).map_err(|error| format!("{error:?}"))?;
                lua.load(bytecode).set_mode(mlua::ChunkMode::Binary)
            } else {
                lua.load(bytes).set_mode(mlua::ChunkMode::Text)
            };
            chunk.set_name(*file).exec().map_err(|error| error.to_string())
        });
        if let Err(error) = outcome {
            eprintln!("tables: {file}.lub: {error}");
        }
    }
}

/// The names of the global tables a state holds and how many entries each has, for looking around.
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
    let lua = run(grf, &["jobidentity", "npcidentity", "jobname", "pcjobnamegender_f", "accessoryid", "accname"]);
    // The translated job names are keyed by `JOBID.<identifier>`, which the files above know under other tables.
    // An identifier none of them has gets a key of its own, so the file still runs to its end.
    let prepared = lua
        .load(
            r#"local unknown = 0
            JOBID = setmetatable({}, { __index = function(_, key)
                local id = (pcJobTbl2 and pcJobTbl2[key]) or (JTtbl and JTtbl[key]) or (jobtbl and jobtbl[key])
                if id == nil then unknown = unknown - 1; id = unknown end
                return id
            end })"#,
        )
        .exec();
    if let Err(error) = prepared {
        eprintln!("tables: {error}");
    }
    let named = run_more(&lua, grf, &["pcjobname"]);
    let job_names = names(named, "PCJobNameTable");

    let mut jobs = serde_json::Map::new();
    for (id, name) in job_names.into_iter().filter(|(id, _)| *id >= 0) {
        jobs.insert(id.to_string(), serde_json::json!({ "name": name }));
    }

    let headgear: serde_json::Map<String, serde_json::Value> = names(&lua, "AccNameTable").into_iter().map(|(id, name)| (id.to_string(), name.into())).collect();
    // The sprite files of NPCs and monsters, by the number the server shows them under.
    let sprites: serde_json::Map<String, serde_json::Value> =
        names(&lua, "JobNameTable").into_iter().filter(|(id, _)| *id >= 0).map(|(id, name)| (id.to_string(), name.into())).collect();
    // The ending of a weapon's sprite files by the kind of weapon (`_단검` for daggers), from `weapontable.lub`.
    let weapons: serde_json::Map<String, serde_json::Value> =
        names(&run(grf, &["weapontable"]), "WeaponNameTable").into_iter().filter(|(id, _)| *id > 0).map(|(id, name)| (id.to_string(), name.into())).collect();
    // The picture of each pet in its window (a file of `유저인터페이스\illust`), by the monster's number: from
    // `petinfo.lub`, whose function `GetPetIllustName_V3` the client asks.
    let pets: serde_json::Map<String, serde_json::Value> =
        names(run_more(&lua, grf, &["petinfo"]), "PetIllustNameTable").into_iter().filter(|(id, _)| *id >= 0).map(|(id, name)| (id.to_string(), name.into())).collect();
    // Its `PetAccActNameTable`: by the accessory a pet wears (an item), the actions it is then drawn with.
    let pet_acts: serde_json::Map<String, serde_json::Value> = names(&lua, "PetAccActNameTable").into_iter().filter(|(id, _)| *id >= 0).map(|(id, name)| (id.to_string(), name.into())).collect();
    serde_json::json!({ "jobs": jobs, "headgear": headgear, "skills": skills(grf), "sprites": sprites, "weapons": weapons, "tree": skill_tree(grf), "pets": pets, "pet_acts": pet_acts, "emotions": emotions(grf), "messages": messages(grf), "hotkeys": hotkeys(grf), "options": options(grf) })
}

/// The client's message table (`msgstringtable.txt`), line by line: what the client says, by the numbers its code
/// knows the lines by.
pub fn messages(grf: &Grf) -> Vec<String> {
    grf.read("data\\msgstringtable.txt").map(|bytes| korean(&bytes).lines().map(|line| line.trim_end_matches('#').to_owned()).collect()).unwrap_or_default()
}

/// Runs more files in a state that already holds others' tables.
fn run_more<'lua>(lua: &'lua Lua, grf: &Grf, files: &[&str]) -> &'lua Lua {
    run_in(lua, grf, files);
    lua
}

/// Runs one of the client's Lua files that lie beside the archives, not in them (`System\\mapInfo_true.lub` and
/// the like): the first of `files` (paths under the client's folder) that is there. Such a file is either
/// bytecode, as the archives' are, or plain text, as the translations are.
fn beside(client_dir: &std::path::Path, files: &[&str]) -> Option<Lua> {
    let (file, bytes) = files.iter().find_map(|file| Some((file, std::fs::read(client_dir.join(file)).ok()?)))?;
    let lua = Lua::new_with(mlua::StdLib::TABLE | mlua::StdLib::STRING | mlua::StdLib::MATH, mlua::LuaOptions::default()).expect("a Lua state");
    let ran = if bytes.starts_with(b"\x1bLua") {
        lunify::unify(&bytes, &lunify::Format::default(), &lunify::Settings::default()).map_err(|error| format!("{error:?}")).and_then(|code| lua.load(code).set_mode(mlua::ChunkMode::Binary).exec().map_err(|error| error.to_string()))
    } else {
        lua.load(bytes).set_mode(mlua::ChunkMode::Text).exec().map_err(|error| error.to_string())
    };
    if let Err(error) = ran {
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
    }
    tables
}
