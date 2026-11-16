//! The client's own tables (which sprite a job or a headgear is), read out of its Lua files.
//!
//! The files are Lua 5.1 bytecode compiled for the 32-bit client. `lunify` rewrites it for the Lua this program
//! embeds, which then simply runs it; the tables are what the files leave in the globals.

use std::collections::BTreeMap;

use mlua::{Lua, Table, Value};

use crate::grf::Grf;

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
        let path = if file.contains('\\') { format!("{ROOT}{file}.lub") } else { format!("{ROOT}datainfo\\{file}.lub") };
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
    let lua = run(grf, &["jobidentity", "npcidentity", "pcjobnamegender_f", "accessoryid", "accname"]);
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
    serde_json::json!({ "jobs": jobs, "headgear": headgear, "skills": skills(grf) })
}

/// Runs more files in a state that already holds others' tables.
fn run_more<'lua>(lua: &'lua Lua, grf: &Grf, files: &[&str]) -> &'lua Lua {
    run_in(lua, grf, files);
    lua
}

/// What an item is called and which pictures are its, by item ID: from the client's `iteminfo.lub`.
pub struct Items(std::collections::HashMap<u32, (String, String)>);

impl Items {
    /// Takes the items' names from the server's item database instead (rAthena's `db/re` or `db/pre-re` folder):
    /// this client's own table is Korean, and the server's names are the ones its players know. Items the
    /// database lacks keep the client's name.
    pub fn with_server_names(mut self, database: &std::path::Path) -> Self {
        let mut named = 0;
        for file in ["item_db_usable.yml", "item_db_equip.yml", "item_db_etc.yml"] {
            let Ok(text) = std::fs::read_to_string(database.join(file)) else { continue };
            // Entries are `- Id: <id>` followed, a few lines on, by `Name: <name>`.
            let mut id = None;
            for line in text.lines() {
                let line = line.trim_start();
                if let Some(value) = line.strip_prefix("- Id:") {
                    id = value.trim().parse::<u32>().ok();
                } else if let Some(name) = line.strip_prefix("Name:") {
                    // The first name after an ID is the item's; later ones belong to other things in the entry.
                    let Some(item) = id.take() else { continue };
                    let name = name.trim().trim_matches(|quote| quote == '"' || quote == '\'').to_owned();
                    self.0.entry(item).or_default().0 = name;
                    named += 1;
                }
            }
        }
        eprintln!("tables: {named} item names from {}", database.display());
        self
    }

    /// Reads the item table. It is large (tens of thousands of items), so the browser asks for the items it
    /// meets instead of getting all of it.
    pub fn load(grf: &Grf) -> Self {
        let lua = run(grf, &["iteminfo"]);
        let mut items = std::collections::HashMap::new();
        if let Ok(table) = lua.globals().get::<Table>("tbl") {
            for (id, item) in table.pairs::<i64, Table>().flatten() {
                let field = |name: &str| match item.get::<Value>(name) {
                    Ok(Value::String(text)) => korean(&text.as_bytes()),
                    _ => String::new(),
                };
                if let Ok(id) = u32::try_from(id) {
                    items.insert(id, (field("identifiedDisplayName"), field("identifiedResourceName")));
                }
            }
        }
        Self(items)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The items with these IDs, as JSON: `{"<id>": {"name": …, "resource": …}}`. IDs the table lacks are left
    /// out.
    pub fn json(&self, ids: impl Iterator<Item = u32>) -> serde_json::Value {
        let found: serde_json::Map<String, serde_json::Value> =
            ids.filter_map(|id| self.0.get(&id).map(|(name, resource)| (id.to_string(), serde_json::json!({ "name": name, "resource": resource })))).collect();
        found.into()
    }
}

/// The skills, by ID: `{"<id>": {"resource": "NV_BASIC", "name": "Basic Skill", "max": 9}}`. The resource names a
/// skill's pictures.
fn skills(grf: &Grf) -> serde_json::Map<String, serde_json::Value> {
    // The skill table names the jobs it speaks of through `JOBID`: any number will do for those.
    let lua = run(grf, &[]);
    if let Err(error) = lua.load("local next = 0; JOBID = setmetatable({}, { __index = function() next = next + 1; return next end })").exec() {
        eprintln!("tables: {error}");
    }
    run_in(&lua, grf, &["skillinfoz\\skillid", "skillinfoz\\skillinfolist"]);
    let mut skills = serde_json::Map::new();
    let Ok(table) = lua.globals().get::<Table>("SKILL_INFO_LIST") else { return skills };
    for (id, skill) in table.pairs::<i64, Table>().flatten() {
        let text = |value: mlua::Result<Value>| match value {
            Ok(Value::String(text)) => korean(&text.as_bytes()),
            _ => String::new(),
        };
        let resource = text(skill.get::<Value>(1));
        if resource.is_empty() {
            continue;
        }
        let max = skill.get::<i64>("MaxLv").unwrap_or(1);
        skills.insert(id.to_string(), serde_json::json!({ "resource": resource, "name": text(skill.get::<Value>("SkillName")), "max": max }));
    }
    skills
}
